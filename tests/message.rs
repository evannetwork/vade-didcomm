use uuid::Uuid;
use vade_didcomm::{AsyncResult, Message};
use utilities::{keypair};

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
    let sign_keypair = keypair::get_signed_keypair();
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
    let sign_keypair = keypair::get_signed_keypair();
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
