use vade::{ResultAsyncifier, Vade};
use vade_didcomm::{AsyncResult, MessageWithBody, ProtocolOutput, VadeDidComm, protocol::DID_EXCHANGE_PROTOCOL_URL, read_db, request::{CommKeyPair, DidcommObj}};

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
    let results = vade.didcomm_send("{}", &exchange_request).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let prepared: ProtocolOutput<MessageWithBody<DidcommObj>> = serde_json::from_str(result)?;
    let db_result = read_db(&format!("comm_keypair_{}_{}", inviter, invitee)).asyncify()?;
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
    inviter: &str,
    invitee: &str,
    message: String,
) -> AsyncResult<()> {
    let results = vade.didcomm_receive("{}", &message).await?;
    let result = results
        .get(0)
        .ok_or("no result")?
        .as_ref()
        .ok_or("no value in result")?;
    let prepared: ProtocolOutput<MessageWithBody<DidcommObj>> = serde_json::from_str(result)?;
    let db_result = read_db(&format!("comm_keypair_{}_{}", invitee, inviter)).asyncify()?;
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

    return Ok(());
}

#[tokio::test]
async fn can_do_key_exchange() -> AsyncResult<()> {
    let mut vade = get_vade().await?;
    let inviter = String::from("did:uknow:d34db33d");
    let invitee = String::from("did:uknow:d34db33f");

    let send_message = send_request(&mut vade, &inviter, &invitee).await?;
    receive_request(&mut vade, &inviter, &invitee, send_message).await?;

    Ok(())
}
