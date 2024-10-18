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
    x3dh,
    xxxdh::Protocol,
};

pub struct Socket {
    ctx: WebviewWindow,
    pub ws_sender: Arc<Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>>,
    ws_rcvr: Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
    pub stream_type: String,
    pub msg_queue: Arc<Mutex<Vec<MsgPayload>>>,
    pub app_handle: tauri::AppHandle,
    pub token: Arc<Mutex<String>>
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

    async fn fetch_bundle(&mut self, user: String, token: String) -> Result<(), util::Error>;
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
            app_handle,
            token: Arc::new(Mutex::new(String::new()))
        }))
    }

    async fn send_msg(&mut self, mut msg: MsgPayload, sk: &str) -> Result<(), util::Error> {
        info!("sending: {:?}", msg.content.clone());

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
        self.ws_sender.lock().await.send(payload).await?;
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
            token: token.clone(),
            author: "me".to_string(),
            recipient: "".to_string(),
        };

        info!("sending: {:?}", msg.clone());

        let json = serde_json::to_string(&msg)?;
        let payload = Message::text(json);

        self.ws_sender.lock().await.send(payload).await?;
        *self.token.lock().await = token;

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
        let token = self.token.clone();

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
                                            if msg.token != "" {
                                                ctx.emit("register_token", msg).unwrap();
                                            }
                                        } else if v.action == "fetch_bundle" {
                                            let x = alice_x3dh(app_handle.clone(), msg, token.lock().await.clone()).await;
                                            let json = serde_json::to_string(&x).unwrap();
                                            let payload = Message::text(json);
                                            ws_sender.lock().await.send(payload).await.unwrap();
                                        } else if v.action == "x3dh" {
                                            bob_x3dh(app_handle.clone(), msg_queue.clone(), msg.clone(), token.lock().await.clone()).await;
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

pub async fn bob_x3dh(app_handle: tauri::AppHandle, msg_queue: Arc<Mutex<Vec<MsgPayload>>>, msg: MsgPayload, token: String){

    let kb = msg.auth.unwrap().keybundle.unwrap();

    let store = app_handle.store_builder("credentials.bin").build();

    let sndr_keybundle = store.get(msg.recipient.clone()).unwrap();
    let sndr_keybundle = serde_json::from_value::<KeyBundle>(sndr_keybundle).unwrap();

    let bob_identity = get_key_pair(sndr_keybundle.identity).unwrap();
    let bob_prekey = get_key_pair(sndr_keybundle.prekey).unwrap();
    let bob_signature = x25519_ristretto::Signature::from_bytes(
        &BASE64_STANDARD
            .decode(sndr_keybundle.signature.public)
            .unwrap(),
    )
    .unwrap();

    

    let mut bob_onetime_key: Option<KeyPairB64> = Option::None;

    for k in sndr_keybundle.onetime_keys{
        if k.public == kb.onetime_keys.get(0).unwrap().public{
            bob_onetime_key = Some(k);
        }
    }

    let bob_onetime_key2 = get_key_pair(bob_onetime_key.clone().unwrap()).unwrap();

    let bob_onetime_key = x25519_ristretto::PublicKey::from_bytes(&BASE64_STANDARD.decode(bob_onetime_key.unwrap().public).unwrap())
    .unwrap();

    let mut bob_protocol =
        Protocol::new(bob_identity, bob_prekey.clone(), bob_signature, Some(vec![bob_onetime_key2]));

    let alice_identity =
        x25519_ristretto::PublicKey::from_bytes(&BASE64_STANDARD.decode(kb.identity.public).unwrap())
            .unwrap();

    let alice_ephemeral_key = x25519_ristretto::PublicKey::from_bytes(&BASE64_STANDARD.decode(kb.ephemeral_key.unwrap().public).unwrap())
    .unwrap();

    let bob_sk = bob_protocol
        .derive_shared_secret(
            &alice_identity,
            &alice_ephemeral_key,
            &bob_onetime_key,
            &BASE64_STANDARD.decode(msg.content.clone().unwrap().nonce).unwrap(),
            &BASE64_STANDARD.decode(msg.content.unwrap().ciphertext).unwrap(),
        )
        .unwrap();

    info!("bob_sk {:?}, author: {}", bob_sk, msg.author.clone());

    // save bob_sk

    let secret_store = app_handle.store_builder("secrets.bin").build();
    secret_store.set(msg.author.clone(), BASE64_STANDARD.encode(bob_sk));
    
    secret_store.save().unwrap();


}

pub async fn alice_x3dh(app_handle: tauri::AppHandle, msg: MsgPayload, token: String) -> MsgPayload {

    let rcvr_keybundle = msg.auth.clone().unwrap().keybundle.unwrap();

    let store = app_handle.store_builder("credentials.bin").build();

    let sndr_keybundle = store.get(msg.recipient.clone()).unwrap();
    let sndr_keybundle = serde_json::from_value::<KeyBundle>(sndr_keybundle).unwrap();

    let alice_identity = get_key_pair(sndr_keybundle.identity).unwrap();
    let alice_prekey = get_key_pair(sndr_keybundle.prekey).unwrap();
    let alice_signature = x25519_ristretto::Signature::from_bytes(
        &BASE64_STANDARD
            .decode(sndr_keybundle.signature.public)
            .unwrap(),
    )
    .unwrap();
    let mut alice_protocol =
        Protocol::new(alice_identity, alice_prekey.clone(), alice_signature, None);

    let bob_identity = x25519_ristretto::PublicKey::from_bytes(
        &BASE64_STANDARD
            .decode(rcvr_keybundle.identity.public)
            .unwrap(),
    )
    .unwrap();
    let bob_prekey = x25519_ristretto::PublicKey::from_bytes(
        &BASE64_STANDARD
            .decode(rcvr_keybundle.prekey.public)
            .unwrap(),
    )
    .unwrap();
    let bob_signature = x25519_ristretto::Signature::from_bytes(
        &BASE64_STANDARD
            .decode(rcvr_keybundle.signature.public)
            .unwrap(),
    )
    .unwrap();

    let bob_one_time_key = x25519_ristretto::PublicKey::from_bytes(
        &BASE64_STANDARD
            .decode(rcvr_keybundle.onetime_keys.get(0).unwrap().clone().public)
            .unwrap(),
    )
    .unwrap();

    let (alice_identity, alice_ephemeral_key, bob_onetime_key, alice_sk, nonce, ciphertext) =
        alice_protocol
            .prepare_init_msg(&bob_identity, &bob_prekey, bob_signature, &bob_one_time_key)
            .unwrap();

    info!("alice_sk: {:?}", alice_sk);

    // save alice_sk

    let secret_store = app_handle.store_builder("secrets.bin").build();
    secret_store.set(msg.auth.clone().unwrap().user, BASE64_STANDARD.encode(alice_sk));
    
    secret_store.save().unwrap();

    use cryptraits::key::KeyPair;

    let generated_kb: KeyBundle = KeyBundle {
        identity: KeyPairB64 {
            public: BASE64_STANDARD.encode(alice_identity.to_vec()),
            private: None,
        },
        prekey: KeyPairB64 {
            public: BASE64_STANDARD.encode(alice_prekey.public().to_vec()),
            private: None,
        },
        signature: KeyPairB64 {
            public: BASE64_STANDARD.encode(alice_signature.to_vec()),
            private: None,
        },
        onetime_keys: vec![KeyPairB64 {
            public: BASE64_STANDARD.encode(bob_onetime_key.to_vec()),
            private: None,
        }],
        ephemeral_key: Some(KeyPairB64 {
            public: BASE64_STANDARD.encode(alice_ephemeral_key.to_vec()),
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
            action: "x3dh".to_string(),
            user: "".to_string(),
            password: "".to_string(),
            keybundle: Some(generated_kb),
            message: "".to_string(),
        }),
        token: token,
        author: msg.recipient,
        recipient: msg.auth.unwrap().user,
    };
    x
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
