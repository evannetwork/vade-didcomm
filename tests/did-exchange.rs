use utilities::keypair::{get_keypair_set};
use vade::{ResultAsyncifier, Vade};
use vade_didcomm::{AsyncResult, VadeDidComm, protocol::DID_EXCHANGE_PROTOCOL_URL, read_db, request::CommKeyPair};

async fn get_vade() -> AsyncResult<Vade> {
    let mut vade = Vade::new();
    let vade_didcomm = VadeDidComm::new().await?;
    vade.register_plugin(Box::from(vade_didcomm));

    Ok(vade)
}

async fn do_request(
    mut vade: Vade,
    inviter: &str,
    invitee: &str,
) -> AsyncResult<()> {
    // let exchange_request = get_request_message(
    //     &inviter,
    //     &invitee,
    //     "http://evan.network",
    // );
    // check if keys were saved in rocks db

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

    let results = vade.didcomm_send(
        "{}",
        &exchange_request,
    ).await?;

    let db_result = read_db(&format!("comm_keypair_{}", invitee)).asyncify()?;
    let _: CommKeyPair = serde_json::from_str(&db_result)?;

    return Ok(());
}

#[tokio::test]
async fn can_do_key_exchange() -> AsyncResult<()> {
    let mut vade = get_vade().await?;
    let sign_keypair = get_keypair_set();
    let inviter = String::from("did:uknow:d34db33d");
    let invitee = String::from("did:uknow:d34db33f");

    do_request(vade, &inviter, &invitee).await?;

    Ok(())
}
