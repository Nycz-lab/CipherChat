//! X3DH protocol implementation.

use std::marker::PhantomData;

use cryptimitives::aead::aes_gcm::Aes256Gcm;
use cryptimitives::key::x25519_ristretto::{PublicKey, SecretKey, Signature};
use cryptraits::key::KeyPair;
use cryptraits::{
    aead::Aead,
    convert::{FromBytes, Len, ToVec},
    kdf::Kdf,
    key_exchange::DiffieHellman,
    signature::Verify,
};
use rand_core::{OsRng, RngCore};

use cryptraits::key::Generate;

use cryptimitives::errors::{AeadError, KdfError, KeyPairError, SignatureError};
use thiserror::Error;

/// X3DH protocol errors.
#[derive(Debug, Error)]
pub enum XxxDhError {
    /// There are no prekeys available. Can't establish exchange.
    #[error("there are no one-time prekeys available")]
    EmptyPrekeyList,

    /// Unknown prekey received.
    #[error("unknown prekey")]
    UnknownPrekey,

    /// Error occurred in the underlying KDF function.
    #[error("{0:?}")]
    KdfError(KdfError),

    /// Error occurred in the underlying keypair.
    #[error("{0:?}")]
    KeypairError(KeyPairError),

    /// Error occurred in the underlying AEAD cipher.
    #[error("{0:?}")]
    AeadError(AeadError),

    /// Error occured in the underlying signature.
    #[error("{0:?}")]
    SignatureError(SignatureError),

    /// Storge related errors.
    #[error(transparent)]
    StorageError(#[from] StorageError),
}

/// Storage related errors
#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum StorageError {
    /// Something went wrong.
    #[error("unknown error")]
    UnknownError,
}

impl From<KdfError> for XxxDhError {
    fn from(e: KdfError) -> Self {
        Self::KdfError(e)
    }
}

impl From<AeadError> for XxxDhError {
    fn from(e: AeadError) -> Self {
        Self::AeadError(e)
    }
}

impl From<SignatureError> for XxxDhError {
    fn from(e: SignatureError) -> Self {
        Self::SignatureError(e)
    }
}

/// `Result` specialized to this crate for convenience. Used for protocol related results.
pub type XxxDhResult<T> = Result<T, XxxDhError>;

/// `Result` specialized to this crate for convenience. Used for storage related results.
pub type StorageResult<T> = Result<T, StorageError>;

pub const PROTOCOL_INFO: &'static str = "X3DH";

/// X3DH Protocol.
pub struct Protocol {
    _sk: cryptimitives::key::x25519_ristretto::KeyPair,
    _esk: cryptimitives::key::x25519_ristretto::KeyPair,
    _sig: Signature,
    _otk: Option<Vec<cryptimitives::key::x25519_ristretto::KeyPair>>,
}

impl Protocol {
    pub fn new(
        identity_keypair: cryptimitives::key::x25519_ristretto::KeyPair,
        prekey_keypair: cryptimitives::key::x25519_ristretto::KeyPair,
        prekey_signature: Signature,
        onetime_keypairs: Option<Vec<cryptimitives::key::x25519_ristretto::KeyPair>>,
    ) -> Self {
        Self {
            _sk: identity_keypair,
            _esk: prekey_keypair,
            _sig: prekey_signature,
            _otk: onetime_keypairs,
        }
    }

    /// Derive secret key and create initial message using receiver's keys.
    pub fn prepare_init_msg(
        &mut self,
        receiver_identity: &PublicKey,
        receiver_prekey: &PublicKey,
        receiver_prekey_signature: Signature,
        receiver_onetime_key: &PublicKey,
    ) -> XxxDhResult<(PublicKey, PublicKey, PublicKey, Vec<u8>, Vec<u8>, Vec<u8>)> {
        receiver_identity
            .verify(&receiver_prekey.to_vec(), &receiver_prekey_signature)
            .unwrap();
        let ephemeral_key: cryptimitives::key::x25519_ristretto::KeyPair =
            cryptimitives::key::x25519_ristretto::KeyPair::generate_with(OsRng).into();

        let sk = self._derive_sk([
            (self._sk.secret(), &receiver_prekey),
            (&ephemeral_key.secret(), &receiver_identity),
            (&ephemeral_key.secret(), &receiver_prekey),
            (&ephemeral_key.secret(), &receiver_onetime_key),
        ])?;

        let mut nonce = vec![0; Aes256Gcm::NONCE_LEN];
        OsRng.fill_bytes(&mut nonce);

        let mut data = self._sk.to_public().to_vec();
        data.extend(&receiver_identity.to_vec());

        let cipher = Aes256Gcm::new(&sk);

        let ciphertext = cipher.encrypt(&nonce, &data, None).unwrap();

        Ok((
            self._sk.to_public(),
            ephemeral_key.to_public(),
            *receiver_onetime_key,
            sk,
            nonce,
            ciphertext,
        ))
    }

    /// Derive secret key from sender's message.
    pub fn derive_shared_secret(
        &mut self,
        sender_identity: &PublicKey,
        sender_ephemeral_key: &PublicKey,
        receiver_onetime_key: &PublicKey,
        nonce: &[u8],
        ciphertext: &[u8],
    ) -> XxxDhResult<Vec<u8>> {
        let identity_secret = self._sk.secret();
        let prekey_secret = self._esk.secret();

        let otk_storage = self._otk.clone().unwrap();

        let onetime_keypair = otk_storage.get(0).unwrap();

        let sk = self._derive_sk([
            (prekey_secret, &sender_identity),
            (&identity_secret, sender_ephemeral_key),
            (prekey_secret, sender_ephemeral_key),
            (onetime_keypair.secret(), sender_ephemeral_key),
        ])?;

        let cipher = Aes256Gcm::new(&sk);
        cipher.decrypt(nonce, ciphertext, None)?;

        Ok(sk)
    }

    /// Derive secret key.
    fn _derive_sk(&self, source_data: [(&SecretKey, &PublicKey); 4]) -> XxxDhResult<Vec<u8>> {
        let mut data = vec![0_u8; <<SecretKey as DiffieHellman>::PK as Len>::LEN];

        for (sk, pk) in source_data {
            data.extend(sk.diffie_hellman(pk).to_vec());
        }

        let h = cryptimitives::kdf::sha256::Kdf::new(
            Some(&vec![0_u8; <<SecretKey as DiffieHellman>::SSK as Len>::LEN]),
            &data,
        );

        let mut sk = vec![0_u8; <<SecretKey as DiffieHellman>::SSK as Len>::LEN];

        h.expand(PROTOCOL_INFO.as_bytes(), &mut sk)?;

        Ok(sk)
    }
}
