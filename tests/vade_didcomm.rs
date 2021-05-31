use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;
use vade::Vade;
use vade_didcomm::{AsyncResult, VadeDidComm, VadeTransport, VadeTransportRedisPubsub};

const REDIS_CONNECTION: &str = "redis://localhost";

#[tokio::test]
async fn can_be_registered_as_plugin() -> AsyncResult<()> {
    get_vade(None, None).await?;

    Ok(())
}

#[tokio::test]
async fn can_send_a_message() -> AsyncResult<()> {
    let (mut vade, _, _) = get_vade(None, None).await?;

    // send message
    vade.run_custom_function("did:evan", "pingpong", r#"{ "transfer": "didcomm" }"#, "{}")
        .await?;

    Ok(())
}

#[tokio::test]
async fn can_reply_to_a_ping_with_a_pong() -> AsyncResult<()> {
    let (mut vade, ping_sender_id, channel) = get_vade(None, None).await?;
    println!("sender: {}", ping_sender_id);

    // start listener to check messages
    let (mut test_transport, _, _) = get_transport(None, Some(channel.to_owned()))?;
    let mut receiver = test_transport.listen().await?;

    // start listener in separate task
    let task = tokio::spawn(async {
        // now start a vade, that will respond to our ping
        let (mut listener_vade, pong_sender_id, _) = get_vade(None, Some(channel)).await.unwrap();
        println!("receiver: {}", pong_sender_id);
        println!("pre listen");
        listener_vade
            .run_custom_function("did:evan", "listen", r#"{ "transfer": "didcomm" }"#, "{}")
            .await
            .unwrap();
        println!("post listen");
    });

    println!("pre sleep");

    sleep(Duration::from_millis(1_000u64)).await;

    // send message
    println!("pre pingpong");
    vade.run_custom_function("did:evan", "pingpong", r#"{ "transfer": "didcomm" }"#, "{}")
        .await?;
    println!("post pingpong");

    // receiver will receive ping message
    loop {
        match receiver.try_next() {
            Ok(Some(value)) => {
                println!("test got: {:?}", &value);
                break;
            }
            Ok(None) => {
                println!("disconnected");
                break;
            }
            Err(_) => {
                sleep(Duration::from_millis(100u64)).await;
            }
        };
    }

    // and send a pong message
    loop {
        match receiver.try_next() {
            Ok(Some(value)) => {
                println!("test got: {:?}", &value);
                break;
            }
            Ok(None) => {
                println!("disconnected");
                break;
            }
            Err(_) => {
                sleep(Duration::from_millis(100u64)).await;
            }
        };
    }

    // cleanup / quit
    task.await?;

    Ok(())
}

async fn get_vade(
    id: Option<String>,
    channel: Option<String>,
) -> AsyncResult<(Vade, String, String)> {
    let mut vade = Vade::new();
    let (vade_didcomm, used_id, used_channel) = get_vade_didcomm(id, channel).await?;
    vade.register_plugin(Box::from(vade_didcomm));

    Ok((vade, used_id, used_channel))
}

async fn get_vade_didcomm(
    id: Option<String>,
    channel: Option<String>,
) -> AsyncResult<(VadeDidComm, String, String)> {
    let (mut transport, used_id, used_channel) = get_transport(id, channel)?;
    transport.listen().await?;
    let vade_didcomm =
        VadeDidComm::new(String::from(""), String::from(""), Box::new(transport)).await?;

    Ok((vade_didcomm, used_id, used_channel))
}

fn get_transport(
    id: Option<String>,
    channel: Option<String>,
) -> AsyncResult<(VadeTransportRedisPubsub, String, String)> {
    let used_id = id.unwrap_or_else(|| Uuid::new_v4().to_simple().to_string());
    let used_channel = channel.unwrap_or_else(|| Uuid::new_v4().to_simple().to_string());
    let transport = VadeTransportRedisPubsub::new(
        used_id.to_owned(),
        String::from(REDIS_CONNECTION),
        used_channel.to_owned(),
    )
    .map_err(|e| e.to_string())?;

    Ok((transport, used_id, used_channel))
}
