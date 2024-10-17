use aes_gcm::Key;
use async_trait::async_trait;
use base64::{prelude::BASE64_STANDARD, Engine};
use cryptimitives::key::{self, ed25519::SecretKey, x25519_ristretto, KeyPair};
use cryptraits::{
    convert::{FromBytes, Len},
    key::Generate,
};
use log::info;
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
    x3dh,
    xxxdh::Protocol,
};

pub struct Socket {
    ctx: WebviewWindow,
    pub ws_sender: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    ws_rcvr: Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
    pub stream_type: String,
    pub msg_queue: Arc<Mutex<Vec<MsgPayload>>>,
    pub app_handle: tauri::AppHandle,
}

#[async_trait]
pub trait SocketFuncs {
    async fn new(
        ctx: WebviewWindow,
        url: String,
        app_handle: tauri::AppHandle,
    ) -> Result<Box<Self>, util::Error>;
    async fn send_msg(&mut self, msg: MsgPayload) -> Result<(), util::Error>;
    async fn recv_msg(&mut self);
    async fn close(&mut self) -> Result<(), Error>;

    async fn login(&mut self, auth: MsgPayload) -> Result<(), util::Error>;
    async fn register(&mut self, auth: MsgPayload, keybundle: KeyBundle)
        -> Result<(), util::Error>;

    async fn fetch_bundle(&mut self, user: String, token: String) -> Result<(), util::Error>;
    async fn x3dh(&mut self, msg: MsgPayload);
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
            ws_sender: ws_sender,
            ws_rcvr: Some(ws_rcvr),
            stream_type: stream_type.to_string(),
            msg_queue: Arc::new(Mutex::new(Vec::new())),
            app_handle,
        }))
    }

    async fn send_msg(&mut self, msg: MsgPayload) -> Result<(), util::Error> {
        info!("sending: {:?}", msg.content.clone());
        let json = serde_json::to_string(&msg)?;
        let payload = Message::text(json);
        self.ws_sender.send(payload).await?;
        Ok(())
    }

    async fn fetch_bundle(&mut self, user: String, token: String) -> Result<(), util::Error> {
        let msg = MsgPayload {
            content: None,
            timestamp: 0,
            auth: Some(OpAuthPayload {
                action: "fetch_bundle".to_string(),
                user: user,
                password: "".to_string(),
                keybundle: None,
                message: "".to_string(),
            }),
            token: token,
            author: "me".to_string(),
            recipient: "".to_string(),
        };

        info!("sending: {:?}", msg.content.clone());

        let json = serde_json::to_string(&msg)?;
        let payload = Message::text(json);

        self.ws_sender.send(payload).await?;

        Ok(())
    }

    async fn login(&mut self, auth: MsgPayload) -> Result<(), util::Error> {
        info!("logging in: {}", auth.auth.clone().unwrap().user.clone());
        let json = serde_json::to_string(&auth)?;
        let payload = Message::text(json);
        self.ws_sender.send(payload).await?;
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
        self.ws_sender.send(payload).await?;
        Ok(())
    }

    async fn x3dh(&mut self, msg: MsgPayload) {

        // let alice_identity = x25519_ristretto::KeyPair::generate_with(OsRng);
        // let alice_prekey = x25519_ristretto::KeyPair::generate_with(OsRng);
        // let alice_signature = alice_identity.sign(&alice_prekey.to_public().to_vec());
        // let mut alice_protocol = Protocol::new(alice_identity, alice_prekey, alice_signature, None);
    }

    async fn recv_msg(&mut self) {
        let mut ws_rcvr = self.ws_rcvr.take().unwrap();
        let ctx = self.ctx.clone();

        let msg_queue = self.msg_queue.clone();
        let app_handle = self.app_handle.clone();

        tokio::spawn(async move {
            while let Some(Ok(msg)) = ws_rcvr.next().await {
                match msg {
                    Message::Text(txt) => {
                        // let msg: MsgPayload = serde_json::from_str(&txt).unwrap();
                        match serde_json::from_str::<MsgPayload>(&txt) {
                            Ok(msg) => {
                                info!("received: {:?}", msg);

                                match msg.clone().auth {
                                    Some(v) => {
                                        if v.action == "register" || v.action == "login" {
                                            if msg.token != "" {
                                                ctx.emit("register_token", msg).unwrap();
                                            }
                                        } else if v.action == "fetch_bundle" {
                                            // self.x3dh(msg);

                                            let rcvr_keybundle =
                                                msg.auth.unwrap().keybundle.unwrap();

                                            let store =
                                                app_handle.store_builder("credentials.bin").build();

                                            let sndr_keybundle = store.get(msg.recipient).unwrap();
                                            info!("test: {}", sndr_keybundle);
                                            let sndr_keybundle =
                                                serde_json::from_value::<KeyBundle>(sndr_keybundle)
                                                    .unwrap();

                                            let alice_identity =
                                                get_key_pair(sndr_keybundle.identity).unwrap();
                                            let alice_prekey =
                                                get_key_pair(sndr_keybundle.prekey).unwrap();
                                            let alice_signature =
                                                x25519_ristretto::Signature::from_bytes(
                                                    &BASE64_STANDARD
                                                        .decode(sndr_keybundle.signature.public)
                                                        .unwrap(),
                                                )
                                                .unwrap();
                                            let mut alice_protocol = Protocol::new(
                                                alice_identity,
                                                alice_prekey.clone(),
                                                alice_signature,
                                                None,
                                            );

                                            let bob_identity =
                                                x25519_ristretto::PublicKey::from_bytes(
                                                    &BASE64_STANDARD
                                                        .decode(rcvr_keybundle.identity.public)
                                                        .unwrap(),
                                                )
                                                .unwrap();
                                            let bob_prekey =
                                                x25519_ristretto::PublicKey::from_bytes(
                                                    &BASE64_STANDARD
                                                        .decode(rcvr_keybundle.prekey.public)
                                                        .unwrap(),
                                                )
                                                .unwrap();
                                            let bob_signature =
                                                x25519_ristretto::Signature::from_bytes(
                                                    &BASE64_STANDARD
                                                        .decode(rcvr_keybundle.signature.public)
                                                        .unwrap(),
                                                )
                                                .unwrap();

                                            let bob_one_time_key =
                                                x25519_ristretto::PublicKey::from_bytes(
                                                    &BASE64_STANDARD
                                                        .decode(
                                                            rcvr_keybundle
                                                                .onetime_keys
                                                                .get(0)
                                                                .unwrap()
                                                                .clone()
                                                                .public,
                                                        )
                                                        .unwrap(),
                                                )
                                                .unwrap();

                                            let (
                                                alice_identity,
                                                alice_ephemeral_key,
                                                bob_onetime_key,
                                                alice_sk,
                                                nonce,
                                                ciphertext,
                                            ) = alice_protocol
                                                .prepare_init_msg(
                                                    &bob_identity,
                                                    &bob_prekey,
                                                    bob_signature,
                                                    &bob_one_time_key,
                                                )
                                                .unwrap();
                                            info!("identity: {:?}\nephemeral_key: {:?}\nbob_one_time_key: {:?}\nalice_sk: {:?}\nnonce: {:?}\nciphertext: {:?}\n", alice_identity,
                                            alice_ephemeral_key, bob_onetime_key, alice_sk, nonce, ciphertext);

                                            use cryptraits::key::KeyPair;

                                            let generated_kb: KeyBundle = KeyBundle {
                                                identity: KeyPairB64 {
                                                    public: BASE64_STANDARD
                                                        .encode(alice_identity.to_vec()),
                                                    private: None,
                                                },
                                                prekey: KeyPairB64 {
                                                    public: BASE64_STANDARD
                                                        .encode(alice_prekey.public().to_vec()),
                                                    private: None,
                                                },
                                                signature: KeyPairB64 {
                                                    public: BASE64_STANDARD
                                                        .encode(alice_signature.to_vec()),
                                                    private: None,
                                                },
                                                onetime_keys: vec![KeyPairB64 {
                                                    public: BASE64_STANDARD
                                                        .encode(bob_onetime_key.to_vec()),
                                                    private: None,
                                                }],
                                                ephemeral_key: Some(KeyPairB64 {
                                                    public: BASE64_STANDARD
                                                        .encode(alice_ephemeral_key.to_vec()),
                                                    private: None,
                                                }),
                                            };

                                            let x = MsgPayload {
                                                content: Some(MsgContent {
                                                    ciphertext: BASE64_STANDARD.encode(ciphertext),
                                                    nonce: BASE64_STANDARD.encode(nonce),
                                                    cleartext: None,
                                                }),
                                                timestamp: 0,
                                                auth: Some(OpAuthPayload {
                                                    action: "".to_string(),
                                                    user: "".to_string(),
                                                    password: "".to_string(),
                                                    keybundle: Some(generated_kb),
                                                    message: "".to_string(),
                                                }),
                                                token: "".to_string(),
                                                author: "".to_string(),
                                                recipient: "".to_string(),
                                            };
                                            info!("crafted message: {:?}", x);
                                            msg_queue.lock().await.push(x);
                                        }
                                    }
                                    None => {
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
        self.ws_sender.close().await?;
        Ok(())
    }
}

pub fn get_key_pair(
    key_pair: KeyPairB64,
) -> Result<cryptimitives::key::KeyPair<x25519_ristretto::SecretKey>, Error> {
    let alice_pub =
        x25519_ristretto::PublicKey::from_bytes(&BASE64_STANDARD.decode(key_pair.public).unwrap())
            .unwrap();
    let alice_priv = x25519_ristretto::SecretKey::from_bytes(
        &BASE64_STANDARD
            .decode(match key_pair.private {
                Some(v) => v,
                None => String::new(),
            })
            .unwrap(),
    )
    .unwrap();
    let mut x = alice_pub.to_vec();
    let mut y = alice_priv.to_vec();
    y.append(&mut x);

    let alice_key_bundle: cryptimitives::key::KeyPair<x25519_ristretto::SecretKey> =
        x25519_ristretto::KeyPair::from_bytes(&y).unwrap();

    Ok(alice_key_bundle)
}
