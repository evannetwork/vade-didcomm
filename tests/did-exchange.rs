use vade::{ResultAsyncifier, Vade};
use vade_didcomm::{AsyncResult, CommKeyPair, MessageWithBody, ProtocolOutput, VadeDidComm, get_com_keypair, helper::DidcommObj, protocol::DID_EXCHANGE_PROTOCOL_URL, read_db};

async fn get_vade() -> AsyncResult<Vade> {
    let mut vade = Vade::new();
    let vade_didcomm = VadeDidComm::new().await?;
    vade.register_plugin(Box::from(vade_didcomm));

    Ok(vade)
}

async fn send_request(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
) -> AsyncResult<String> {
    let exchange_request = format!(
        r#"{{
            "type": "{}/request",
            "service_endpoint": "https://evan.network",
            "from": "{}",
            "to": ["{}"]
        }}"#,
        DID_EXCHANGE_PROTOCOL_URL,
        sender,
        receiver
    );
    let results = vade.didcomm_send("{}", &exchange_request).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let prepared: ProtocolOutput<MessageWithBody<DidcommObj>> = serde_json::from_str(result)?;
    let db_result = read_db(&format!("comm_keypair_{}_{}", sender, receiver)).asyncify()?;
    let comm_keypair: CommKeyPair = serde_json::from_str(&db_result)?;

    let pub_key = prepared.metadata
        .get("pub_key")
        .ok_or("send didcomm request does not return pub_key")?
        .to_owned();
    let secret_key = prepared.metadata
        .get("secret_key")
        .ok_or("send didcomm request does not return secret_key")?
        .to_owned();
    let target_pub_key = prepared.metadata
        .get("target_pub_key")
        .ok_or("send didcomm request does not return target_pub_key")?
        .to_owned();

    assert_eq!(target_pub_key, comm_keypair.target_pub_key);
    assert_eq!(pub_key, comm_keypair.pub_key);
    assert_eq!(secret_key, comm_keypair.secret_key);

    return Ok(serde_json::to_string(&prepared.message)?);
}

async fn receive_request(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    message: String,
) -> AsyncResult<()> {
    let results = vade.didcomm_receive("{}", &message).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let received: ProtocolOutput<MessageWithBody<DidcommObj>> = serde_json::from_str(result)?;
    let comm_keypair = get_com_keypair(receiver, sender).asyncify()?;

    let pub_key = received.metadata
        .get("pub_key")
        .ok_or("send didcomm request does not return pub_key")?
        .to_owned();
    let secret_key = received.metadata
        .get("secret_key")
        .ok_or("send didcomm request does not return secret_key")?
        .to_owned();
    let target_pub_key = received.metadata
        .get("target_pub_key")
        .ok_or("send didcomm request does not return target_pub_key")?
        .to_owned();

    assert_eq!(target_pub_key, comm_keypair.target_pub_key);
    assert_eq!(pub_key, comm_keypair.pub_key);
    assert_eq!(secret_key, comm_keypair.secret_key);

    return Ok(());
}

async fn send_response(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
) -> AsyncResult<String> {
    let exchange_response = format!(
        r#"{{
            "type": "{}/response",
            "service_endpoint": "https://evan.network",
            "from": "{}",
            "to": ["{}"]
        }}"#,
        DID_EXCHANGE_PROTOCOL_URL,
        sender,
        receiver
    );
    let results = vade.didcomm_send("{}", &exchange_response).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let prepared: ProtocolOutput<MessageWithBody<DidcommObj>> = serde_json::from_str(result)?;

    return Ok(serde_json::to_string(&prepared.message)?);
}

async fn receive_response(
    vade: &mut Vade,
    sender: &str,
    receiver: &str,
    message: String,
) -> AsyncResult<()> {
    let results = vade.didcomm_receive("{}", &message).await?;
    let _ = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let comm_keypair_sender = get_com_keypair(sender, receiver).asyncify()?;
    let comm_keypair_receiver = get_com_keypair(receiver, sender).asyncify()?;

    assert_eq!(comm_keypair_sender.target_pub_key, comm_keypair_receiver.pub_key);

    return Ok(());
}

#[tokio::test]
async fn can_do_key_exchange() -> AsyncResult<()> {
    let mut vade = get_vade().await?;
    let user_1_did = String::from("did:uknow:d34db33d");
    let user_2_did = String::from("did:uknow:d34db33f");

    let request_message = send_request(&mut vade, &user_1_did, &user_2_did).await?;
    receive_request(&mut vade, &user_1_did, &user_2_did, request_message).await?;

    let response_message = send_response(&mut vade, &user_2_did, &user_1_did).await?;
    receive_response(&mut vade, &user_2_did, &user_1_did, response_message).await?;

    Ok(())
}
