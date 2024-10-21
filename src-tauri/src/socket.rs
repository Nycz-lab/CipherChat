use aes_gcm::Key;
use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine};
use cryptimitives::{aead::aes_gcm::Aes256Gcm, key::{self, ed25519::SecretKey, x25519_ristretto, KeyPair}};
use cryptraits::{
    aead::Aead, convert::{FromBytes, Len}, key::Generate
};
use log::info;
use rand_core::{OsRng, RngCore};
use tauri::{Emitter, WebviewWindow, Window};

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::{fs::File, io::BufReader, path::Path, sync::Arc};
use tauri_plugin_store::StoreExt;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_rustls::rustls::{
    self,
    crypto::cipher,
    pki_types::{pem::PemObject, CertificateDer, TrustAnchor},
    RootCertStore,
};
use tokio_tungstenite::{
    connect_async_tls_with_config,
    tungstenite::{Error, Message},
    Connector, MaybeTlsStream, WebSocketStream,
};

use cryptraits::convert::ToVec;

use crate::{
    util::{self, KeyBundle, KeyPairB64, MsgContent, MsgPayload, OpAuthPayload},
    x3dh::{self, alice_x3dh, bob_x3dh},
    xxxdh::Protocol,
};

pub struct Socket {
    ctx: WebviewWindow,
    pub ws_sender: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
    ws_rcvr: Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
    pub stream_type: String,
    pub msg_queue: Arc<Mutex<Vec<MsgPayload>>>,
    pub app_handle: tauri::AppHandle
}

#[async_trait]
pub trait SocketFuncs {
    async fn new(
        ctx: WebviewWindow,
        url: String,
        app_handle: tauri::AppHandle,
    ) -> Result<Box<Self>, util::Error>;
    async fn send_msg(&mut self, msg: MsgPayload, sk: &str) -> Result<(), util::Error>;
    async fn recv_msg(&mut self);
    async fn close(&mut self) -> Result<(), Error>;

    async fn login(&mut self, auth: MsgPayload) -> Result<(), util::Error>;
    async fn register(&mut self, auth: MsgPayload, keybundle: KeyBundle)
        -> Result<(), util::Error>;

    async fn fetch_bundle(&mut self, user: String) -> Result<(), util::Error>;
}

#[async_trait]
impl SocketFuncs for Socket {
    async fn new(
        ctx: WebviewWindow,
        url: String,
        app_handle: tauri::AppHandle,
    ) -> Result<Box<Self>, util::Error> {
        let tls_config = match Path::new("rootCA.crt").exists() {
            true => {
                let mut root_cert_store = RootCertStore::empty();

                let cert_der = CertificateDer::from_pem_file("rootCA.crt").unwrap();
                root_cert_store.add(cert_der).unwrap();

                info!("using provided root ca");

                rustls::ClientConfig::builder()
                    .with_root_certificates(root_cert_store)
                    .with_no_client_auth()
            }
            false => {
                info!("defaulting to os native certs");
                rustls_platform_verifier::tls_config()
            }
        };

        let connector = Connector::Rustls(Arc::new(tls_config));

        let (ws_stream, _) =
            connect_async_tls_with_config(url, None, false, Some(connector)).await?;

        let stream_type = match ws_stream.get_ref() {
            MaybeTlsStream::Plain(_) => "unencrypted",
            MaybeTlsStream::Rustls(_) => "TLS",
            _ => "not available",
        };

        let (ws_sender, ws_rcvr) = ws_stream.split();

        Ok(Box::new(Socket {
            ctx: ctx,
            ws_sender: Arc::new(Mutex::new(ws_sender)),
            ws_rcvr: Some(ws_rcvr),
            stream_type: stream_type.to_string(),
            msg_queue: Arc::new(Mutex::new(Vec::new())),
            app_handle
        }))
    }

    async fn send_msg(&mut self, mut msg: MsgPayload, sk: &str) -> Result<(), util::Error> {
        info!("sending: {:?}", msg.content.clone());

        let payload = encrypt_msg(msg, sk).await?;
        self.ws_sender.lock().await.send(payload).await?;
        Ok(())
    }

    async fn fetch_bundle(&mut self, user: String) -> Result<(), util::Error> {
        let msg = MsgPayload {
            content: None,
            timestamp: 0,
            auth: Some(OpAuthPayload {
                action: "fetch_bundle".to_string(),
                user: user,
                password: "".to_string(),
                keybundle: None,
                message: "".to_string(),
                success: None
            }),
            message_id: "".to_string(),
            author: "me".to_string(),
            recipient: "".to_string(),
        };

        info!("sending: {:?}", msg.clone());

        let json = serde_json::to_string(&msg)?;
        let payload = Message::text(json);

        self.ws_sender.lock().await.send(payload).await?;

        Ok(())
    }

    async fn login(&mut self, auth: MsgPayload) -> Result<(), util::Error> {
        info!("logging in: {}", auth.auth.clone().unwrap().user.clone());
        let json = serde_json::to_string(&auth)?;
        let payload = Message::text(json);
        self.ws_sender.lock().await.send(payload).await?;
        Ok(())
    }

    async fn register(
        &mut self,
        mut auth: MsgPayload,
        keybundle: KeyBundle,
    ) -> Result<(), util::Error> {
        info!(
            "registering as: {}",
            auth.auth.clone().unwrap().user.clone()
        );
        // auth.auth.unwrap().keybundle = Some(keybundle);
        if let Some(auth_data) = auth.auth.as_mut() {
            auth_data.keybundle = Some(keybundle);
        }
        let json = serde_json::to_string(&auth)?;
        let payload = Message::text(json);
        self.ws_sender.lock().await.send(payload).await?;
        Ok(())
    }

    async fn recv_msg(&mut self) {
        let mut ws_rcvr = self.ws_rcvr.take().unwrap();
        let ws_sender = self.ws_sender.clone();
        let ctx = self.ctx.clone();

        let msg_queue = self.msg_queue.clone();
        let app_handle = self.app_handle.clone();


        tokio::spawn(async move {
            while let Some(Ok(msg)) = ws_rcvr.next().await {
                match msg {
                    Message::Text(txt) => {
                        // let msg: MsgPayload = serde_json::from_str(&txt).unwrap();
                        match serde_json::from_str::<MsgPayload>(&txt) {
                            Ok(mut msg) => {
                                info!("received: {:?}", msg);

                                match msg.clone().auth {
                                    Some(v) => {
                                        if v.action == "register" || v.action == "login" {
                                            match v.success{
                                                Some(v) => {
                                                    if v == true{
                                                        ctx.emit("register_token", msg).unwrap();
                                                    }
                                                },
                                                None => return,
                                            };
                                            
                                        } else if v.action == "fetch_bundle" {
                                            let x = alice_x3dh(app_handle.clone(), msg).await;
                                            let json = serde_json::to_string(&x).unwrap();
                                            let payload = Message::text(json);
                                            ws_sender.lock().await.send(payload).await.unwrap();
                                            info!("sent x3dh payload");

                                            let z = msg_queue.lock().await;

                                            for msg in (*z).iter() {

                                                let store = app_handle.store_builder("secrets.bin").build();
                                                let sk = store.get(&msg.recipient).unwrap();

                                                let payload = encrypt_msg(msg.clone(), sk.as_str().unwrap()).await.unwrap();
                                                ws_sender.lock().await.send(payload).await.unwrap();
                                                
                                            }


                                            
                                        } else if v.action == "x3dh" {
                                            bob_x3dh(app_handle.clone(), msg_queue.clone(), msg.clone()).await;
                                        }
                                    }
                                    None => {
                                        let store = app_handle.store_builder("secrets.bin").build();
                                        let x = store.get(&msg.author).unwrap();
                                        let sk = x.as_str().unwrap();

                                        let sk = BASE64_STANDARD.decode(sk).unwrap();

                                        let msg_content = msg.content.as_mut().unwrap();

                                        let nonce = BASE64_STANDARD.decode(&msg_content.nonce).unwrap();
                                        let ciphertext = BASE64_STANDARD.decode(&msg_content.ciphertext).unwrap();

                                        let cipher = Aes256Gcm::new(&sk);
                                        let cleartext = cipher.decrypt(&nonce, &ciphertext, None).unwrap();

                                        let cleartext = String::from_utf8(cleartext).unwrap();

                                        msg_content.cleartext = Some(cleartext);

                                        info!("decrypted msg: {:?}", msg.clone());

                                        ctx.emit("msg", msg).unwrap();
                                    }
                                };
                            }
                            Err(e) => {
                                error!("received wrong data: {:?}", e);
                            }
                        }
                    }
                    Message::Binary(_) => todo!(),
                    Message::Ping(_) => todo!(),
                    Message::Pong(_) => todo!(),
                    Message::Close(_) => {
                        info!("conn closed")
                    }
                    Message::Frame(_) => todo!(),
                }
            }

            info!("Connection closed?");
            ctx.emit("connection_closed", {}).unwrap();
        });
    }

    async fn close(&mut self) -> Result<(), Error> {
        self.ws_sender.lock().await.close().await?;
        Ok(())
    }
}


async fn encrypt_msg(mut msg: MsgPayload, sk: &str) -> Result<Message, util::Error> {

    let sk = BASE64_STANDARD.decode(sk).unwrap();

    let mut nonce = vec![0; Aes256Gcm::NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    
    let cipher = Aes256Gcm::new(&sk);

    let msg_content = msg.content.as_mut().unwrap();
    let cleartext = msg_content.clone().cleartext.unwrap();

    let ciphertext = cipher.encrypt(&nonce, cleartext.as_bytes(), None).unwrap();

    msg_content.cleartext = None;
    msg_content.ciphertext = BASE64_STANDARD.encode(ciphertext);
    msg_content.nonce = BASE64_STANDARD.encode(nonce);

    let json = serde_json::to_string(&msg)?;
    let payload = Message::text(json);
    
    Ok(payload)
}
