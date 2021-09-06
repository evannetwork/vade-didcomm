use k256::elliptic_curve::rand_core::OsRng;
use base58::FromBase58;
use x25519_dalek::{PublicKey, StaticSecret};
use ed25519_dalek::{Keypair};
use arrayref::array_ref;

pub struct KeyPairSet {
    pub user1_pub: PublicKey,
    pub user1_secret: StaticSecret,
    pub user2_pub: PublicKey,
    pub user2_secret: StaticSecret,
    pub sign_keypair: Keypair,
    pub sign_keypair2: Keypair,
}

pub fn get_keypair_set() -> KeyPairSet {

    let alice_private = "6QN8DfuN9hjgHgPvLXqgzqYE3jRRGRrmJQZkd5tL8paR".from_base58().unwrap();
    let bobs_private = "HBTcN2MrXNRj9xF9oi8QqYyuEPv3JLLjQKuEgW9oxVKP".from_base58().unwrap();

    let alice_secret_key: StaticSecret = StaticSecret::from(array_ref!(alice_private, 0, 32).to_owned());
    let bob_secret_key: StaticSecret = StaticSecret::from(array_ref!(bobs_private, 0, 32).to_owned());

    let alice_public: PublicKey = (&alice_secret_key).into();
    let bob_public: PublicKey = (&bob_secret_key).into();

    let sign_keypair: ed25519_dalek::Keypair = ed25519_dalek::Keypair::generate(&mut OsRng);
    let sign_keypair2: ed25519_dalek::Keypair = ed25519_dalek::Keypair::generate(&mut OsRng);



    return KeyPairSet {
        user1_pub: alice_public,
        user1_secret: alice_secret_key,
        user2_pub: bob_public,
        user2_secret: bob_secret_key,
        sign_keypair: sign_keypair,
        sign_keypair2: sign_keypair2,
    }
}
