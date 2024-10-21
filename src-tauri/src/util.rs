use std::str::Utf8Error;

use base64::DecodeError;

use crate::crypt::AesGcmErrorWrapper;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Utf8(#[from] Utf8Error),
    #[error(transparent)]
    Decode(#[from] DecodeError),
    #[error(transparent)]
    Aes(#[from] AesGcmErrorWrapper),
    #[error(transparent)]
    Join(#[from] tokio::task::JoinError),
    #[error(transparent)]
    Tung(#[from] tokio_tungstenite::tungstenite::Error),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),

    #[error("An error occurred: {0}")]
    CustomError(String),
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MsgPayload{
    pub content: Option<MsgContent>,
    pub timestamp: u64,
    pub auth: Option<OpAuthPayload>,
    pub message_id: String,
    pub author: String,
    pub recipient: String
  }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyBundle{
    pub identity: KeyPairB64,
    pub prekey: KeyPairB64,
    pub signature: KeyPairB64,
    pub onetime_keys: Vec<KeyPairB64>,
    pub ephemeral_key: Option<KeyPairB64>
  }

impl KeyBundle {
    pub fn strip(&mut self) {
        self.identity.strip();
        self.prekey.strip();
        self.signature.strip();
        for otk in &mut self.onetime_keys {
            otk.strip();
        }
        match &mut self.ephemeral_key{
            Some(v) => v.strip(),
            None => (),
        };
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyPairB64 {
    pub public: String,
    pub private: Option<String>,
}

impl KeyPairB64 {
    pub fn strip(&mut self) {
        self.private = None;
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct OpAuthPayload{
    pub action: String,
    pub user: String,
    pub password: String,
    pub keybundle: Option<KeyBundle>,
    pub message: String,
    pub success: Option<bool>,
  }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MsgContent{
  pub ciphertext: String,
  pub nonce: String,
  pub cleartext: Option<String>
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ConnectionInfo {
    pub host: String,
    pub stream_type: String,
}
