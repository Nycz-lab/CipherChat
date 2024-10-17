use aes_gcm::Key;
use async_trait::async_trait;
use cryptimitives::key;
use log::info;
use tauri::{Emitter, WebviewWindow, Window};

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::{fs::File, io::BufReader, path::Path, sync::Arc};
use tokio::net::TcpStream;
use tokio_rustls::rustls::{self, pki_types::{pem::PemObject, CertificateDer, TrustAnchor}, RootCertStore};
use tokio_tungstenite::{
    connect_async_tls_with_config,
    tungstenite::{Error, Message},
    Connector, MaybeTlsStream, WebSocketStream,
};

use crate::util::{self, KeyBundle, MsgPayload};

pub struct Socket {
    ctx: WebviewWindow,
    pub ws_sender: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    ws_rcvr: Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>,
    pub stream_type: String,
}

#[async_trait]
pub trait SocketFuncs {
    async fn new(ctx: WebviewWindow, url: String) -> Result<Box<Self>, util::Error>;
    async fn send_msg(&mut self, msg: MsgPayload) -> Result<(), util::Error>;
    async fn recv_msg(&mut self);
    async fn close(&mut self) -> Result<(), Error>;

    async fn login(&mut self, auth: MsgPayload) -> Result<(), util::Error>;
    async fn register(&mut self, auth: MsgPayload, keybundle: KeyBundle)
        -> Result<(), util::Error>;
}

#[async_trait]
impl SocketFuncs for Socket {
    async fn new(ctx: WebviewWindow, url: String) -> Result<Box<Self>, util::Error> {


        let tls_config = match Path::new("rootCA.crt").exists(){
            true => {
                let mut root_cert_store = RootCertStore::empty();

                let cert_der = CertificateDer::from_pem_file("rootCA.crt").unwrap();
                root_cert_store.add(cert_der).unwrap();

                info!("using provided root ca");

                rustls::ClientConfig::builder()
                    .with_root_certificates(root_cert_store)
                    .with_no_client_auth()
            },
            false => {
                info!("defaulting to os native certs");
                rustls_platform_verifier::tls_config()
            },
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
        }))
    }

    async fn send_msg(&mut self, msg: MsgPayload) -> Result<(), util::Error> {
        info!("sending: {}", msg.content.clone());
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

    async fn recv_msg(&mut self) {
        let mut ws_rcvr = self.ws_rcvr.take().unwrap();
        let ctx = self.ctx.clone();
        tokio::spawn(async move {
            while let Some(Ok(msg)) = ws_rcvr.next().await {
                match msg {
                    Message::Text(txt) => {
                        // let msg: MsgPayload = serde_json::from_str(&txt).unwrap();
                        match serde_json::from_str::<MsgPayload>(&txt) {
                            Ok(msg) => {
                                info!("received: {:?}", msg);

                                if msg.content_type == "auth" {
                                    if msg.token != "" {
                                        ctx.emit("register_token", msg).unwrap();
                                    }
                                } else {
                                    ctx.emit("msg", msg).unwrap();
                                }
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
