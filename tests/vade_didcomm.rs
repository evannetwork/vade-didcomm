use k256::elliptic_curve::rand_core::OsRng;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
use vade::Vade;
use vade_didcomm::{
    AsyncResult,
    DidcommReceiveResult,
    VadeDidComm,
};
use x25519_dalek::{PublicKey, StaticSecret};

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

async fn get_vade(
    id: Option<String>,
    channel: Option<String>,
) -> AsyncResult<Vade> {
    let mut vade = Vade::new();
    let vade_didcomm = get_vade_didcomm(id, channel).await?;
    vade.register_plugin(Box::from(vade_didcomm));

    Ok(vade)
}

async fn get_vade_didcomm(
    id: Option<String>,
    channel: Option<String>,
) -> AsyncResult<VadeDidComm> {
    let vade_didcomm = VadeDidComm::new(String::from(""), String::from("")).await?;

    Ok(vade_didcomm)
}

#[tokio::test]
async fn can_be_registered_as_plugin() -> AsyncResult<()> {
    get_vade(None, None).await?;

    Ok(())
}

#[tokio::test]
async fn can_send_a_message() -> AsyncResult<()> {
    let mut vade = get_vade(None, None).await?;

    // send message
    vade.run_custom_function("did:evan", "pingpong", r#"{ "transfer": "didcomm" }"#, "{}")
        .await?;

    Ok(())
}

// #[tokio::test]
// async fn can_reply_to_a_ping_with_a_pong() -> AsyncResult<()> {
//     let (mut vade, ping_sender_id, channel) = get_vade(None, None).await?;
//     println!("sender: {}", ping_sender_id);

//     // start listener to check messages
//     let (mut test_transport, _, _) = get_transport(None, Some(channel.to_owned()))?;
//     let mut receiver = test_transport.listen().await?;

//     // start listener in separate task
//     let task = tokio::spawn(async {
//         // now start a vade, that will respond to our ping
//         let (mut listener_vade, pong_sender_id, _) = get_vade(None, Some(channel)).await.unwrap();
//         println!("receiver: {}", pong_sender_id);
//         println!("pre listen");
//         listener_vade
//             .run_custom_function("did:evan", "listen", r#"{ "transfer": "didcomm" }"#, "{}")
//             .await
//             .unwrap();
//         println!("post listen");
//     });

//     println!("pre sleep");

//     sleep(Duration::from_millis(1_000u64)).await;

//     // send message
//     println!("pre pingpong");
//     vade.run_custom_function("did:evan", "pingpong", r#"{ "transfer": "didcomm" }"#, "{}")
//         .await?;
//     println!("post pingpong");

//     // receiver will receive ping message
//     loop {
//         match receiver.try_next() {
//             Ok(Some(value)) => {
//                 println!("test got: {:?}", &value);
//                 break;
//             }
//             Ok(None) => {
//                 println!("disconnected");
//                 break;
//             }
//             Err(_) => {
//                 sleep(Duration::from_millis(100u64)).await;
//             }
//         };
//     }

//     // and send a pong message
//     loop {
//         match receiver.try_next() {
//             Ok(Some(value)) => {
//                 println!("test got: {:?}", &value);
//                 break;
//             }
//             Ok(None) => {
//                 println!("disconnected");
//                 break;
//             }
//             Err(_) => {
//                 sleep(Duration::from_millis(100u64)).await;
//             }
//         };
//     }

//     // cleanup / quit
//     task.await?;

//     Ok(())
// }

// #[tokio::test]
// async fn can_prepare_didcomm_message_for_sending() -> AsyncResult<()> {
//     let (mut vade, _, _) = get_vade(None, None).await?;

//     let alice_secret = StaticSecret::new(OsRng);
//     let bob_secret = StaticSecret::new(OsRng);
//     let bob_public = PublicKey::from(&bob_secret);

//     let ek = alice_secret.diffie_hellman(&bob_public);

//     let sign_keypair = ed25519_dalek::Keypair::generate(&mut OsRng);

//     let options = format!(
//         r#"{{
//             "encryptionKey": {:?},
//             "signKeypair": {:?}
//         }}"#,
//         &ek.as_bytes(),
//         &sign_keypair.to_bytes(),
//     );

//     let payload = format!(
//         r#"{{
//             "to": [ "did::xyz:34r3cu403hnth03r49g03" ],
//             "body": {},
//             "custom1": "ichi",
//             "custom2": "ni",
//             "custom3": "san"
//         }}"#,
//         serde_json::to_string(EXAMPLE_DID_DOCUMENT)?
//     );
//     let results = vade.didcomm_send(&options, &payload).await?;
//     match results.get(0) {
//         Some(Some(value)) => {
//             println!("got didcomm msg: {}", &value);
//         }
//         _ => {
//             return Err(Box::from("invalid result from didcomm_send"));
//         }
//     };

//     Ok(())
// }

// #[tokio::test]
// async fn can_decrypt_received_messages() -> AsyncResult<()> {
//     let (mut vade, _, _) = get_vade(None, None).await?;

//     let alice_secret = StaticSecret::new(OsRng);
//     let alice_public = PublicKey::from(&alice_secret);
//     let bob_secret = StaticSecret::new(OsRng);
//     let bob_public = PublicKey::from(&bob_secret);

//     let ek = alice_secret.diffie_hellman(&bob_public);
//     let rk = bob_secret.diffie_hellman(&alice_public);

//     let sign_keypair = ed25519_dalek::Keypair::generate(&mut OsRng);

//     let options = format!(
//         r#"{{
//             "encryptionKey": {:?},
//             "signKeypair": {:?}
//         }}"#,
//         &ek.as_bytes(),
//         &sign_keypair.to_bytes(),
//     );

//     let payload = format!(
//         r#"{{
//             "to": [ "did::xyz:34r3cu403hnth03r49g03" ],
//             "body": {},
//             "custom1": "nyuu"
//         }}"#,
//         serde_json::to_string(EXAMPLE_DID_DOCUMENT)?
//     );
//     let results = vade.didcomm_send(&options, &payload).await?;
//     match results.get(0) {
//         Some(Some(value)) => {
//             let options = format!(
//                 r#"{{
//                     "decryptionKey": {:?},
//                     "signPublic": {:?}
//                 }}"#,
//                 &rk.as_bytes(),
//                 &sign_keypair.public.to_bytes(),
//             );
//             let results = vade.didcomm_receive(&options, &value).await?;
//             let result = results
//                 .get(0)
//                 .ok_or("no result")?
//                 .as_ref()
//                 .ok_or("no value in result")?;
//             let parsed: DidcommReceiveResult = serde_json::from_str(result)?;
//             assert_eq!(EXAMPLE_DID_DOCUMENT, parsed.body);
//         }
//         _ => {
//             return Err(Box::from("invalid result from didcomm_send"));
//         }
//     };

//     Ok(())
// }
