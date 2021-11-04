use arrayref::array_ref;
use base58::FromBase58;
use ed25519_dalek::Keypair;
use k256::elliptic_curve::rand_core::OsRng;
use vade_didcomm::datatypes::{DidCommOptions, EncryptionKeys, SigningKeys};
use x25519_dalek::{PublicKey, StaticSecret};
pub struct KeyPairSet {
    pub user1_pub: PublicKey,
    pub user1_secret: StaticSecret,
    pub user1_did: String,
    pub user2_pub: PublicKey,
    pub user2_secret: StaticSecret,
    pub user2_did: String,
    pub sign_keypair: Keypair,
    pub sign_keypair2: Keypair,
    pub sender_options: DidCommOptions,
    pub receiver_options: DidCommOptions,
    pub receiver_options_stringified: String,
    pub sender_options_stringified: String,
    pub sender_signing_options: DidCommOptions,
    pub receiver_signing_options: DidCommOptions,
    pub receiver_signing_options_stringified: String,
    pub sender_signing_options_stringified: String,
}

pub fn get_keypair_set() -> KeyPairSet {
    let alice_private = "6QN8DfuN9hjgHgPvLXqgzqYE3jRRGRrmJQZkd5tL8paR"
        .from_base58()
        .unwrap();
    let bobs_private = "HBTcN2MrXNRj9xF9oi8QqYyuEPv3JLLjQKuEgW9oxVKP"
        .from_base58()
        .unwrap();

    let alice_did = String::from("did:key:z6MkiTBz1ymuepAQ4HEHYSF1H8quG5GLVVQR3djdX3mDooWp");
    let bob_did = String::from("did:key:z6MkjchhfUsD6mmvni8mCdXHw216Xrm9bQe2mBH1P5RDjVJG");

    let alice_secret_key: StaticSecret =
        StaticSecret::from(array_ref!(alice_private, 0, 32).to_owned());
    let bob_secret_key: StaticSecret =
        StaticSecret::from(array_ref!(bobs_private, 0, 32).to_owned());

    let alice_public: PublicKey = (&alice_secret_key).into();
    let bob_public: PublicKey = (&bob_secret_key).into();

    let sign_keypair: ed25519_dalek::Keypair = ed25519_dalek::Keypair::generate(&mut OsRng);
    let sign_keypair2: ed25519_dalek::Keypair = ed25519_dalek::Keypair::generate(&mut OsRng);

    let sender_options: DidCommOptions = DidCommOptions {
        encryption_keys: Some(EncryptionKeys {
            encryption_my_secret: alice_secret_key.to_bytes(),
            encryption_others_public: Some(bob_public.to_bytes()),
        }),
        signing_keys: Some(SigningKeys {
            signing_my_secret: Some(sign_keypair.secret.to_bytes()),
            signing_others_public: Some(sign_keypair2.public.to_bytes()),
        }),
        skip_protocol_handling: Some(false),
    };
    let sender_options_stringified =
        serde_json::to_string(&sender_options).unwrap_or_else(|_| "{}".to_string());

    let sender_signing_options: DidCommOptions = DidCommOptions {
        encryption_keys: None,
        signing_keys: Some(SigningKeys {
            signing_my_secret: Some(sign_keypair.secret.to_bytes()),
            signing_others_public: Some(sign_keypair2.public.to_bytes()),
        }),
        skip_protocol_handling: Some(false),
    };
    let sender_signing_options_stringified =
        serde_json::to_string(&sender_signing_options).unwrap_or_else(|_| "{}".to_string());

    let receiver_options: DidCommOptions = DidCommOptions {
        encryption_keys: Some(EncryptionKeys {
            encryption_my_secret: bob_secret_key.to_bytes(),
            encryption_others_public: Some(alice_public.to_bytes()),
        }),
        signing_keys: Some(SigningKeys {
            signing_my_secret: Some(sign_keypair2.secret.to_bytes()),
            signing_others_public: Some(sign_keypair.public.to_bytes()),
        }),
        skip_protocol_handling: Some(false),
    };
    let receiver_options_stringified =
        serde_json::to_string(&receiver_options).unwrap_or_else(|_| "{}".to_string());

    let receiver_signing_options: DidCommOptions = DidCommOptions {
        encryption_keys: None,
        signing_keys: Some(SigningKeys {
            signing_my_secret: Some(sign_keypair2.secret.to_bytes()),
            signing_others_public: Some(sign_keypair.public.to_bytes()),
        }),
        skip_protocol_handling: Some(false),
    };
    let receiver_signing_options_stringified =
        serde_json::to_string(&receiver_signing_options).unwrap_or_else(|_| "{}".to_string());

    return KeyPairSet {
        user1_pub: alice_public,
        user1_secret: alice_secret_key,
        user1_did: alice_did,
        user2_did: bob_did,
        user2_pub: bob_public,
        user2_secret: bob_secret_key,
        sign_keypair: sign_keypair,
        sign_keypair2: sign_keypair2,
        receiver_options,
        receiver_options_stringified,
        sender_options,
        sender_options_stringified,
        receiver_signing_options,
        receiver_signing_options_stringified,
        sender_signing_options,
        sender_signing_options_stringified,
    };
}
