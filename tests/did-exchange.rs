use didcomm_rs::Message;
use utilities::keypair::{get_keypair_set};
use vade::{ResultAsyncifier, Vade};
use vade_didcomm::{AsyncResult, EncryptedMessage, ProtocolOutput, VadeDidComm, protocol::DID_EXCHANGE_PROTOCOL_URL, read_db, request::CommKeyPair};

async fn get_vade() -> AsyncResult<Vade> {
    let mut vade = Vade::new();
    let vade_didcomm = VadeDidComm::new().await?;
    vade.register_plugin(Box::from(vade_didcomm));

    Ok(vade)
}

async fn send_request(
    vade: &mut Vade,
    inviter: &str,
    invitee: &str,
) -> AsyncResult<String> {
    let exchange_request = format!(
        r#"{{
            "type": "{}/request",
            "service_endpoint": "https://evan.network",
            "from": "{}",
            "to": ["{}"]
        }}"#,
        DID_EXCHANGE_PROTOCOL_URL,
        inviter,
        invitee
    );
    println!("============> 1");
    let results = vade.didcomm_send("{}", &exchange_request).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    println!("============> 1.1: {}", result);
    let prepared: ProtocolOutput<Message> = serde_json::from_str(result)?;
    println!("============> 2");
    let db_result = read_db(&format!("comm_keypair_{}_{}", inviter, invitee)).asyncify()?;
    println!("============> 3");
    let comm_keypair: CommKeyPair = serde_json::from_str(&db_result)?;
    println!("============> 4");

    let encoded_pub_key = prepared.metadata
        .get("encoded_pub_key")
        .ok_or("send didcomm request does not return encoded_pub_key")?
        .to_owned();
    let encoded_secret_key = prepared.metadata
        .get("encoded_secret_key")
        .ok_or("send didcomm request does not return encoded_secret_key")?
        .to_owned();
    let encoded_target_pub_key = prepared.metadata
        .get("encoded_target_pub_key")
        .ok_or("send didcomm request does not return encoded_target_pub_key")?
        .to_owned();
    println!("============> 5");

    assert_eq!(encoded_target_pub_key, comm_keypair.encoded_target_pub_key);
    assert_eq!(encoded_pub_key, comm_keypair.encoded_pub_key);
    assert_eq!(encoded_secret_key, comm_keypair.encoded_secret_key);

    return Ok(serde_json::to_string(&prepared.message)?);
}

async fn receive_request(
    vade: &mut Vade,
    inviter: &str,
    invitee: &str,
    message: String,
) -> AsyncResult<()> {
    println!("------------- {}", &message);
    let results = vade.didcomm_receive("{}", &message).await?;

    return Ok(());
}

#[tokio::test]
async fn can_do_key_exchange() -> AsyncResult<()> {
    let mut vade = get_vade().await?;
    let inviter = String::from("did:uknow:d34db33d");
    let invitee = String::from("did:uknow:d34db33f");

    let send_message = send_request(&mut vade, &inviter, &invitee).await?;
    // receive_request(&mut vade, &inviter, &invitee, send_message).await?;

    Ok(())
}
