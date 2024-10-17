//! Basic example.

use std::path::PathBuf;

use aes_gcm::Key;
use base64::{prelude::BASE64_STANDARD, Engine};
use cryptimitives::{aead, kdf::sha256, key::x25519_ristretto};
use cryptraits::{
    convert::{FromBytes, ToVec},
    key::KeyPair,
    signature::Sign,
};
use rand_core::OsRng;
use serde_json::json;
use tauri::{Manager, Wry};
use tauri_plugin_store::StoreExt;
// use tauri_plugin_store::{with_store, StoreCollection};

use cryptraits::key::Generate;


use crate::{util::{KeyBundle, KeyPairB64, MsgPayload}, xxxdh::Protocol};

pub fn get_keybundle(app_handle: tauri::AppHandle, auth: MsgPayload) -> KeyBundle {

    let identity: cryptimitives::key::KeyPair<x25519_ristretto::SecretKey> = x25519_ristretto::KeyPair::generate_with(OsRng);
    let prekey = x25519_ristretto::KeyPair::generate_with(OsRng);
    let signature = identity.sign(&prekey.to_public().to_vec());

    let mut ot_kp: Vec<KeyPairB64> = Vec::new();

    for _i in 0..10 {
        let onetime_keypair = x25519_ristretto::KeyPair::generate_with(OsRng);

        let kp = KeyPairB64 {
            public: BASE64_STANDARD.encode(onetime_keypair.public().to_vec()),
            private: Some(BASE64_STANDARD.encode(onetime_keypair.secret().to_vec())),
        };

        ot_kp.push(kp);
    }

    let mut public_kb: KeyBundle = KeyBundle {
        identity: KeyPairB64 {
            public: BASE64_STANDARD.encode(identity.public().to_vec()),
            private: Some(BASE64_STANDARD.encode(identity.secret().to_vec())),
        },
        prekey: KeyPairB64 {
            public: BASE64_STANDARD.encode(prekey.public().to_vec()),
            private: Some(BASE64_STANDARD.encode(prekey.secret().to_vec())),
        },
        signature: KeyPairB64 {
            public: BASE64_STANDARD.encode(signature.to_vec()),
            private: None,
        },
        onetime_keys: ot_kp,
        ephemeral_key: None,
    };

    let store = app_handle.store_builder("credentials.bin").build();
    store.set(auth.auth.unwrap().user, json!(public_kb));

    store.save().unwrap();

    public_kb.strip();

    public_kb
}

#[test]
fn check_secret_sharing_x3dh() {
    // Instantiate Alice protocol.

    let alice_identity = x25519_ristretto::KeyPair::generate_with(OsRng);
    let alice_prekey = x25519_ristretto::KeyPair::generate_with(OsRng);
    let alice_signature: x25519_ristretto::Signature = alice_identity.sign(&alice_prekey.to_public().to_vec());
    let mut alice_protocol = Protocol::new(alice_identity, alice_prekey, alice_signature, None);

    // Instantiate Bob protocol.

    let onetime_keypair = x25519_ristretto::KeyPair::generate_with(OsRng);
    let bob_identity = x25519_ristretto::KeyPair::generate_with(OsRng);
    let bob_prekey = x25519_ristretto::KeyPair::generate_with(OsRng);
    let bob_signature = bob_identity.clone().sign(&bob_prekey.to_public().to_vec());
    let mut bob_protocol = Protocol::new(
        bob_identity.clone(),
        bob_prekey.clone(),
        bob_signature,
        Some(vec![onetime_keypair.clone()]),
    );

    // Derive shared secret for Alice and prepare message for Bob.

    let bob_identity = bob_identity;
    let bob_prekey = bob_prekey;
    let bob_signature = bob_signature;
    let onetime_key = onetime_keypair;

    let (alice_identity, alice_ephemeral_key, bob_onetime_key, alice_sk, nonce, ciphertext) =
        alice_protocol
            .prepare_init_msg(bob_identity.public(), bob_prekey.public(), bob_signature, onetime_key.public())
            .unwrap();

    // Derive shared secret for Bob using Alice credentials.

    let bob_sk = bob_protocol
        .derive_shared_secret(
            &alice_identity,
            &alice_ephemeral_key,
            &bob_onetime_key,
            &nonce,
            &ciphertext,
        )
        .unwrap();

    assert_eq!(alice_sk, bob_sk);
}
