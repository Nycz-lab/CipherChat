use aes_gcm::{
    aead::{generic_array::GenericArray, Aead, AeadCore, KeyInit, OsRng},
    aes::cipher::typenum, // Or `Aes128Gcm`
    Aes256Gcm,
    Key,
    Nonce,
};

use sha256::digest;
use std::{
    error::Error as StdError,
    str::{self},
};

use base64::{engine::general_purpose, Engine as _};
// create the error type that represents all errors possible in our program
use crate::Error;

// we must manually implement serde::Serialize
impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

// Wrapper struct for aes_gcm::Error
#[derive(Debug)]
pub struct AesGcmErrorWrapper(pub aes_gcm::Error);

// Implementing the StdError trait for AesGcmErrorWrapper
impl StdError for AesGcmErrorWrapper {}

// Implementing the Display trait for AesGcmErrorWrapper
impl std::fmt::Display for AesGcmErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AesGcm Error: {}", self.0)
    }
}

#[test]
fn check_symmetry() {
    let key = String::from("VerySecureyPassword123LmaoKek");
    let og_txt = String::from("The cake was a lie!");
    let cipthertext = encrypt(key.to_string(), og_txt.to_string()).unwrap();
    let plaintext = decrypt(key.to_string(), cipthertext.to_string()).unwrap();

    println!(
        "Original: {}\nKey: {}\nCiphertext: {}\nProcessed Result: {}",
        &og_txt, &key, &cipthertext, &plaintext
    );
    assert_eq!(&plaintext, &og_txt);
}

pub fn encrypt(key: String, plaintext: String) -> Result<String, Error> {
    let hash = digest(key.to_string()).into_bytes();
    let crypt_key = Key::<Aes256Gcm>::from_slice(&hash[..32]);

    let plaintext_bytes = plaintext.into_bytes();

    let cipher = Aes256Gcm::new(&crypt_key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
    let mut ciphertext = match cipher.encrypt(&nonce, plaintext_bytes.as_ref()) {
        Ok(text) => text,
        Err(error) => return Err(Error::Aes(AesGcmErrorWrapper(error))),
    };

    let mut nonce_cp = nonce.to_vec();
    ciphertext.append(&mut nonce_cp);

    let b64 = general_purpose::STANDARD.encode(ciphertext);

    return Ok(b64);
}

pub fn decrypt(key: String, ciphertext: String) -> Result<String, Error> {
    let hash = digest(key.to_string()).into_bytes();
    let crypt_key = Key::<Aes256Gcm>::from_slice(&hash[..32]);

    let mut ciphertext_bytes = general_purpose::STANDARD.decode(ciphertext.to_string())?;

    let mut nonce: [u8; 12] = [0; 12];
    nonce.copy_from_slice(&ciphertext_bytes[ciphertext_bytes.len() - 12..]);
    ciphertext_bytes.truncate(ciphertext_bytes.len() - 12);

    let nonce_generic: Nonce<typenum::U12> = GenericArray::from(nonce);

    let cipher = Aes256Gcm::new(&crypt_key);
    let plaintext_bytes = match cipher.decrypt(&nonce_generic, ciphertext_bytes.as_ref()) {
        Ok(bytes) => bytes,
        Err(error) => return Err(Error::Aes(AesGcmErrorWrapper(error))),
    };

    let plaintext = str::from_utf8(&plaintext_bytes)?;

    return Ok(plaintext.to_string());
}
