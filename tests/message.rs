use uuid::Uuid;
use vade_didcomm::{AsyncResult, Message};
use k256::elliptic_curve::rand_core::OsRng;
use vade::Vade;

use x25519_dalek::{PublicKey, StaticSecret};

struct KeyPairSet {
    pub user1_pub: PublicKey,
    pub user1_secret: StaticSecret,
    pub user1_shared: x25519_dalek::SharedSecret,
    pub user2_pub: PublicKey,
    pub user2_secret: StaticSecret,
    pub user2_shared: x25519_dalek::SharedSecret,
    pub sign_keypair: ed25519_dalek::Keypair,
}

fn get_signed_keypair() -> KeyPairSet {
    let user1_secret = StaticSecret::new(OsRng);
    let user1_public = PublicKey::from(&user1_secret);
    let user2_secret = StaticSecret::new(OsRng);
    let user2_public = PublicKey::from(&user2_secret);

    let user1_shared = user1_secret.diffie_hellman(&user2_public);
    let user2_shared = user2_secret.diffie_hellman(&user1_public);

    let sign_keypair: ed25519_dalek::Keypair = ed25519_dalek::Keypair::generate(&mut OsRng);

    return KeyPairSet {
        user1_pub: user1_public,
        user1_secret: user1_secret,
        user1_shared: user1_shared,
        user2_pub: user2_public,
        user2_secret: user2_secret,
        user2_shared: user2_shared,
        sign_keypair: sign_keypair,
    }
}

#[tokio::test]
async fn can_instantiate_message() -> AsyncResult<()> {
    let id = Uuid::new_v4().to_simple().to_string();
    let message_string = format!(
        r#"{{
            "type": "https://didcomm.org/trust_ping/1.0/ping",
            "id": {:?},
            "body": "test"
        }}"#,
        &id,
    );
    let message = Message::from_string(&message_string)?;
    assert_eq!(message.r#type.ok_or("Could not get type")?, "https://didcomm.org/trust_ping/1.0/ping");

    Ok(())
}

#[tokio::test]
async fn can_encrypt_message() -> AsyncResult<()> {
    let sign_keypair = get_signed_keypair();
    let payload = format!(
        r#"{{
            "to": [ "did::xyz:34r3cu403hnth03r49g03" ],
            "body": "test",
            "custom1": "ichi",
            "custom2": "ni",
            "custom3": "san"
        }}"#,
    );

    let encrypted = Message::encrypt(
        &payload,
        sign_keypair.user1_shared.as_bytes(),
        &sign_keypair.sign_keypair.to_bytes(),
    )?;
    match encrypted {
        Some(value) => {
            println!("got didcomm msg: {}", &value);
        }
        _ => {
            return Err(Box::from("invalid result from Message::encrypt"));
        }
    };

    Ok(())
}


#[tokio::test]
async fn can_decrypt_message() -> AsyncResult<()> {
    let sign_keypair = get_signed_keypair();
    let payload = format!(
        r#"{{
            "to": [ "did::xyz:34r3cu403hnth03r49g03" ],
            "body": "test",
            "custom1": "ichi",
            "custom2": "ni",
            "custom3": "san"
        }}"#,
    );

    let encrypted = Message::encrypt(
        &payload,
        sign_keypair.user1_shared.as_bytes(),
        &sign_keypair.sign_keypair.to_bytes(),
    )?;

    let decrypted = Message::decrypt(
        &encrypted.ok_or("Could not encrypt")?,
        sign_keypair.user2_shared.as_bytes(),
        &sign_keypair.sign_keypair.public.to_bytes(),
    )?;

    assert_eq!(decrypted.body.body, "test");

    Ok(())
}
