use utilities::keypair::get_signed_keypair;
use vade::Vade;
use vade_didcomm::{AsyncResult, Message, VadeDidComm};

const EXAMPLE_DID_DOCUMENT: &str = r#"{
    "@context": "https://w3id.org/did/v1",
    "id": "did:uknow:d34db33f",
    "publicKey": [
        {
            "id": "did:uknow:d34db33f#cooked",
            "type": "Secp256k1VerificationKey2018",
            "owner": "did:uknow:d34db33f",
            "publicKeyHex": "b9c5714089478a327f09197987f16f9e5d936e8a"
        }
    ],
    "authentication": [
        {
            "type": "Secp256k1SignatureAuthentication2018",
            "publicKey": "did:uknow:d34db33f#cooked"
        }
    ],
    "service": [],
    "created": ""
}"#;

async fn get_vade() -> AsyncResult<Vade> {
    let mut vade = Vade::new();
    let vade_didcomm = get_vade_didcomm().await?;
    vade.register_plugin(Box::from(vade_didcomm));

    Ok(vade)
}

async fn get_vade_didcomm() -> AsyncResult<VadeDidComm> {
    let vade_didcomm = VadeDidComm::new().await?;

    Ok(vade_didcomm)
}

fn get_send_options(
    encryption_key: &x25519_dalek::SharedSecret,
    sign_keypair: &ed25519_dalek::Keypair,
) -> String {
    let options = format!(
        r#"{{
            "encryptionKey": {:?},
            "signKeypair": {:?}
        }}"#,
        &encryption_key.as_bytes(),
        &sign_keypair.to_bytes(),
    );

    return options;
}

fn get_receive_options(
    decryption_key: &x25519_dalek::SharedSecret,
    sign_keypair: &ed25519_dalek::Keypair,
) -> String {
    let options = format!(
        r#"{{
            "decryptionKey": {:?},
            "signPublic": {:?}
        }}"#,
        &decryption_key.as_bytes(),
        &sign_keypair.public.to_bytes()
    );

    return options;
}

#[tokio::test]
async fn can_be_registered_as_plugin() -> AsyncResult<()> {
    get_vade().await?;

    Ok(())
}

#[tokio::test]
async fn can_prepare_didcomm_message_for_sending() -> AsyncResult<()> {
    let mut vade = get_vade().await?;

    let sign_keypair = get_signed_keypair();
    let options = get_send_options(&sign_keypair.user1_shared, &sign_keypair.sign_keypair);
    let payload = format!(
        r#"{{
            "to": [ "did::xyz:34r3cu403hnth03r49g03" ],
            "body": {},
            "custom1": "ichi",
            "custom2": "ni",
            "custom3": "san"
        }}"#,
        serde_json::to_string(EXAMPLE_DID_DOCUMENT)?
    );
    let results = vade.didcomm_send(&options, &payload).await?;
    match results.get(0) {
        Some(Some(value)) => {
            println!("got didcomm msg: {}", &value);
        }
        _ => {
            return Err(Box::from("invalid result from didcomm_send"));
        }
    };

    Ok(())
}

#[tokio::test]
async fn can_decrypt_received_messages() -> AsyncResult<()> {
    let mut vade = get_vade().await?;

    let sign_keypair = get_signed_keypair();
    let options = get_send_options(&sign_keypair.user1_shared, &sign_keypair.sign_keypair);

    let payload = format!(
        r#"{{
            "to": [ "did::xyz:34r3cu403hnth03r49g03" ],
            "body": {},
            "custom1": "nyuu"
        }}"#,
        serde_json::to_string(EXAMPLE_DID_DOCUMENT)?
    );
    let results = vade.didcomm_send(&options, &payload).await?;

    match results.get(0) {
        Some(Some(value)) => {
            let options = get_receive_options(&sign_keypair.user2_shared, &sign_keypair.sign_keypair);
            let results = vade.didcomm_receive(&options, &value).await?;
            let result = results
                .get(0)
                .ok_or("no result")?
                .as_ref()
                .ok_or("no value in result")?;
            let parsed: Message = serde_json::from_str(result)?;
            assert_eq!(EXAMPLE_DID_DOCUMENT, parsed.body);
        }
        _ => {
            return Err(Box::from("invalid result from didcomm_send"));
        }
    };

    Ok(())
}


// #[tokio::test]
// async fn can_reply_to_a_ping_with_a_pong() -> AsyncResult<()> {
//     let mut vade = get_vade().await?;

//     let sign_keypair = get_signed_keypair();
//     let options1 = get_send_options(sign_keypair.user1_shared, &sign_keypair.sign_keypair);
//     let options2 = get_didcomm_options(sign_keypair.user2_shared, &sign_keypair.sign_keypair);

//     vade.didcomm_send(options, payload);


//     Ok(())
// }
