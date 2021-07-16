use k256::elliptic_curve::rand_core::OsRng;
use x25519_dalek::{PublicKey, StaticSecret};

pub struct KeyPairSet {
    pub user1_pub: PublicKey,
    pub user1_secret: StaticSecret,
    pub user1_shared: x25519_dalek::SharedSecret,
    pub user2_pub: PublicKey,
    pub user2_secret: StaticSecret,
    pub user2_shared: x25519_dalek::SharedSecret,
    pub sign_keypair: ed25519_dalek::Keypair,
    pub sign_keypair2: ed25519_dalek::Keypair,
}

pub fn get_keypair_set() -> KeyPairSet {
    let user1_secret = StaticSecret::new(OsRng);
    let user1_public = PublicKey::from(&user1_secret);
    let user2_secret = StaticSecret::new(OsRng);
    let user2_public = PublicKey::from(&user2_secret);

    let user1_shared = user1_secret.diffie_hellman(&user2_public);
    let user2_shared = user2_secret.diffie_hellman(&user1_public);

    let sign_keypair: ed25519_dalek::Keypair = ed25519_dalek::Keypair::generate(&mut OsRng);
    let sign_keypair2: ed25519_dalek::Keypair = ed25519_dalek::Keypair::generate(&mut OsRng);

    return KeyPairSet {
        user1_pub: user1_public,
        user1_secret: user1_secret,
        user1_shared: user1_shared,
        user2_pub: user2_public,
        user2_secret: user2_secret,
        user2_shared: user2_shared,
        sign_keypair: sign_keypair,
        sign_keypair2: sign_keypair2,
    }
}
