//! Basic example.

use std::{path::PathBuf, sync::Arc};

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
use tokio::sync::Mutex;

use crate::{
    util::{get_store_path, KeyBundle, KeyPairB64, MsgContent, MsgPayload, OpAuthPayload},
    xxxdh::Protocol,
    Error, HOMESERVER,
};

pub async fn get_keybundle(app_handle: tauri::AppHandle, auth: MsgPayload) -> KeyBundle {
    let identity: cryptimitives::key::KeyPair<x25519_ristretto::SecretKey> =
        x25519_ristretto::KeyPair::generate_with(OsRng);
    let prekey = x25519_ristretto::KeyPair::generate_with(OsRng);
    let signature = identity.sign(&prekey.to_public().to_vec());

    let mut ot_kp: Vec<KeyPairB64> = Vec::new();

    for _i in 0..100 {
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
    

    let store = app_handle
        .store_builder(get_store_path("credentials.bin").await)
        .build().unwrap();
    store.set(auth.auth.unwrap().user, json!(public_kb));

    store.save().unwrap();

    public_kb.strip();

    public_kb
}

pub async fn bob_x3dh(
    app_handle: tauri::AppHandle,
    msg_queue: Arc<Mutex<Vec<MsgPayload>>>,
    msg: MsgPayload,
) {
    let kb = msg.auth.unwrap().keybundle.unwrap();

    let store = app_handle
        .store_builder(get_store_path("credentials.bin").await)
        .build().unwrap();

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

    for k in sndr_keybundle.onetime_keys {
        if k.public == kb.onetime_keys.get(0).unwrap().public {
            bob_onetime_key = Some(k);
        }
    }

    let bob_onetime_key2 = get_key_pair(bob_onetime_key.clone().unwrap()).unwrap();

    let bob_onetime_key = x25519_ristretto::PublicKey::from_bytes(
        &BASE64_STANDARD
            .decode(bob_onetime_key.unwrap().public)
            .unwrap(),
    )
    .unwrap();

    let mut bob_protocol = Protocol::new(
        bob_identity,
        bob_prekey.clone(),
        bob_signature,
        Some(vec![bob_onetime_key2]),
    );

    let alice_identity = x25519_ristretto::PublicKey::from_bytes(
        &BASE64_STANDARD.decode(kb.identity.public).unwrap(),
    )
    .unwrap();

    let alice_ephemeral_key = x25519_ristretto::PublicKey::from_bytes(
        &BASE64_STANDARD
            .decode(kb.ephemeral_key.unwrap().public)
            .unwrap(),
    )
    .unwrap();

    let bob_sk = bob_protocol
        .derive_shared_secret(
            &alice_identity,
            &alice_ephemeral_key,
            &bob_onetime_key,
            &BASE64_STANDARD
                .decode(msg.content.clone().unwrap().nonce)
                .unwrap(),
            &BASE64_STANDARD
                .decode(msg.content.unwrap().ciphertext)
                .unwrap(),
        )
        .unwrap();

    info!("bob_sk {:?}, author: {}", bob_sk, msg.author.clone());

    // save bob_sk

    info!("saving in {}", format!("{}/secrets.bin", msg.recipient));

    let secret_store = app_handle
        .store_builder(get_store_path(&format!("{}/secrets.bin", msg.recipient)).await)
        .build().unwrap();
    secret_store.set(msg.author.clone(), BASE64_STANDARD.encode(bob_sk));

    secret_store.save().unwrap();
}

pub async fn alice_x3dh(app_handle: tauri::AppHandle, msg: MsgPayload) -> MsgPayload {
    let rcvr_keybundle = msg.auth.clone().unwrap().keybundle.unwrap();

    let store = app_handle
        .store_builder(get_store_path("credentials.bin").await)
        .build().unwrap();

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

    info!("saving in {}", format!("{}/secrets.bin", msg.recipient));

    let secret_store = app_handle
        .store_builder(get_store_path(&format!("{}/secrets.bin", msg.recipient)).await)
        .build().unwrap();
    secret_store.set(
        msg.auth.clone().unwrap().user,
        BASE64_STANDARD.encode(alice_sk),
    );

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
            success: Some(true),
        }),
        message_id: "".to_string(),
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

#[test]
fn check_secret_sharing_x3dh() {
    // Instantiate Alice protocol.

    let alice_identity = x25519_ristretto::KeyPair::generate_with(OsRng);
    let alice_prekey = x25519_ristretto::KeyPair::generate_with(OsRng);
    let alice_signature: x25519_ristretto::Signature =
        alice_identity.sign(&alice_prekey.to_public().to_vec());
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
            .prepare_init_msg(
                bob_identity.public(),
                bob_prekey.public(),
                bob_signature,
                onetime_key.public(),
            )
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
