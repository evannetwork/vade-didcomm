use serde::{Deserialize, Serialize};
use vade_didcomm::{SyncResult, EncryptedMessage, MessageWithBody, decrypt_message, encrypt_message};
use utilities::{keypair};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TestBody {
    test: bool,
}

#[test]
fn can_encrypt_message() -> SyncResult<()> {
    let sign_keypair = keypair::get_keypair_set();
    let payload = format!(
        r#"{{
            "body": {{"test": true}},
            "custom1": "ichi",
            "custom2": "ni",
            "custom3": "san",
            "to": [ "did::xyz:34r3cu403hnth03r49g03" ],
            "type": "test"
        }}"#,
    );

    let encrypted = encrypt_message(
        &payload,
        sign_keypair.user1_shared.as_bytes(),
        &sign_keypair.sign_keypair.to_bytes(),
    )?;
    let _: EncryptedMessage = serde_json::from_str(&encrypted)?;

    Ok(())
}

#[test]
fn can_decrypt_message() -> SyncResult<()> {
    let sign_keypair = keypair::get_keypair_set();
    let payload = format!(
        r#"{{
            "body": {{"test": true}},
            "custom1": "ichi",
            "custom2": "ni",
            "custom3": "san",
            "to": [ "did::xyz:34r3cu403hnth03r49g03" ],
            "type": "test"
        }}"#,
    );

    let encrypted = encrypt_message(
        &payload,
        sign_keypair.user1_shared.as_bytes(),
        &sign_keypair.sign_keypair.to_bytes(),
    )?;

    let decrypted = decrypt_message(
        &encrypted,
        sign_keypair.user2_shared.as_bytes(),
        &sign_keypair.sign_keypair.public.to_bytes(),
    )?;

    let decryped_parsed: MessageWithBody<TestBody> = serde_json::from_str(&decrypted)?;

    assert_eq!(decryped_parsed.body.ok_or("body not available")?.test, true);

    Ok(())
}
