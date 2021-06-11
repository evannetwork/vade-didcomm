use crate::{
    utils::AsyncResult,
    vade_transport::{Message, ProtocolPayload, VadeTransport},
};
use async_trait::async_trait;
use futures::channel::{mpsc, oneshot};
use redis::Commands;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{error::Error, thread, time::Duration};
use tokio::time::sleep;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedisPubSubPayload {
    #[serde(flatten)]
    pub protocol_payload: ProtocolPayload,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RedisPubSubMetadata {
    pub sender: String,
}

pub struct VadeTransportRedisPubsub {
    channel: String,
    client: redis::Client,
    id: String,
    redis_connection: String,
}

impl VadeTransportRedisPubsub {
    #[allow(dead_code)]
    pub fn new(
        id: String,
        redis_connection: String,
        channel: String,
    ) -> Result<VadeTransportRedisPubsub, Box<dyn Error>> {
        match env_logger::try_init() {
            Ok(_) | Err(_) => (),
        };
        Ok(VadeTransportRedisPubsub {
            channel,
            client: redis::Client::open(redis_connection.to_owned())?,
            id,
            redis_connection,
        })
    }
}

#[async_trait]
impl VadeTransport for VadeTransportRedisPubsub {
    async fn handle_message(&mut self, _message_obj: Message) -> AsyncResult<()> {
        log::warn!("handled request");

        Ok(())
    }

    async fn listen(&mut self) -> AsyncResult<mpsc::UnboundedReceiver<Message>> {
        log::debug!("starting pubsub listener");

        let (msg_sender, msg_receiver) = mpsc::unbounded();
        let (subscription_sender, mut subscription_receiver) = oneshot::channel::<()>();
        let redis_connection = self.redis_connection.to_owned();
        let channel_id = self.channel.to_owned();
        let local_id = self.id.to_owned();

        let get_messages_detached = move || {
            match get_message_in_loop(
                redis_connection,
                channel_id,
                subscription_sender,
                local_id,
                msg_sender,
            ) {
                Ok(_) => {
                    log::info!("listener gracefully shut down")
                }
                Err(err) => {
                    log::warn!("listener ran into an error: {}", &err)
                }
            };
        };

        thread::spawn(get_messages_detached);

        loop {
            sleep(Duration::from_millis(1u64)).await;
            let is_subscribed = subscription_receiver.try_recv()?;
            match is_subscribed {
                Some(()) => {
                    log::debug!("subscribed to pubsub");
                    break;
                }
                _ => {
                    log::debug!("waiting for pubsub subscription");
                }
            }
        }

        Ok(msg_receiver)
    }

    async fn send_message(&self, message: &Message) -> AsyncResult<()> {
        log::trace!("sending message");
        let mut con = self.client.get_connection()?;
        let mut to_send = message.clone();
        let empty_object = String::from("{}");
        let metadata_str = message.metadata.as_ref().unwrap_or(&empty_object);
        let mut metadata_obj: Value = serde_json::from_str(metadata_str)?;
        if metadata_obj.get("sender").is_none() {
            let metadata_map = metadata_obj
                .as_object_mut()
                .ok_or("metadata object parsing failed")?;
            metadata_map.insert(String::from("sender"), json!(self.id.to_owned()));
            to_send.metadata = Some(serde_json::to_string(metadata_map)?);
        }
        let json = serde_json::to_string(&to_send)?;
        con.publish(String::from(&self.channel), json)?;

        Ok(())
    }
}

fn get_message_in_loop(
    redis_connection: String,
    channel_id: String,
    subscription_sender: oneshot::Sender<()>,
    local_id: String,
    mut msg_sender: mpsc::UnboundedSender<Message>,
) -> AsyncResult<()> {
    let client = redis::Client::open(redis_connection)?;
    let mut con = client.get_connection()?;
    let mut pubsub = con.as_pubsub();
    pubsub.subscribe(channel_id.to_owned())?;
    subscription_sender
        .send(())
        .map_err(|_| "failed to send connection confirmation")?;
    loop {
        let msg = pubsub
            .get_message()
            .map_err(|e| format!("could not get pubsub message; {}", &e))?;
        let received: String = msg.get_payload()?;
        log::debug!("received message: {:?}", &received);
        let message_obj = serde_json::from_str::<Message>(&received)?;

        log::trace!("pubsub got: {:?}", &message_obj);

        // forward message if not send from self
        let metadata = &message_obj.metadata.as_ref().ok_or("missing metadata")?;
        let metadata = serde_json::from_str::<RedisPubSubMetadata>(&metadata)?;
        log::trace!(
            "payload.sender != local_id, {}, {}",
            &metadata.sender,
            &local_id
        );
        if metadata.sender != local_id {
            log::trace!("forwarding message");
            msg_sender.start_send(message_obj.clone())?;
        }

        // allow to stop listener for tests
        #[cfg(test)]
        let protocol_payload = serde_json::from_str::<ProtocolPayload>(&message_obj.payload)?;
        #[cfg(test)]
        if protocol_payload.protocol == "stop_listening" {
            log::info!("stopping redis pubsub listener");
            pubsub.unsubscribe(channel_id.to_owned())?;
            ()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use uuid::Uuid;

    const REDIS_CONNECTION: &str = "redis://localhost";

    fn get_vade_transport(
        id: &str,
        channel: Option<String>,
    ) -> AsyncResult<VadeTransportRedisPubsub> {
        VadeTransportRedisPubsub::new(
            String::from(id),
            String::from(REDIS_CONNECTION),
            channel.unwrap_or_else(|| Uuid::new_v4().to_simple().to_string()),
        )
        .map_err(|err| Box::from(err.to_string()))
    }

    #[tokio::test]
    async fn can_send_a_message() -> AsyncResult<()> {
        let vade_didcomm = get_vade_transport("sender", None)?;
        let payload = ProtocolPayload {
            protocol: String::from("stop_listening"),
            step: String::from("stop_listening"),
        };
        let message = Message::new(payload, None as Option<()>).map_err(|e| e.to_string())?;
        vade_didcomm.send_message(&message).await?;

        Ok(())
    }

    #[tokio::test]
    async fn can_start_a_listener() -> AsyncResult<()> {
        let channel = Uuid::new_v4().to_simple().to_string();
        let mut vade_didcomm_listener = get_vade_transport("receiver", Some(channel.to_owned()))?;

        // start listener
        vade_didcomm_listener.listen().await?;

        // stop it
        let vade_didcomm_sender = get_vade_transport("sender", Some(channel.to_owned()))?;
        let payload = ProtocolPayload {
            protocol: String::from("stop_listening"),
            step: String::from("stop_listening"),
        };
        let message = Message::new(payload, None as Option<()>).map_err(|e| e.to_string())?;
        vade_didcomm_sender.send_message(&message).await?;

        Ok(())
    }

    #[tokio::test]
    async fn can_receive_a_message() -> AsyncResult<()> {
        let channel = Uuid::new_v4().to_simple().to_string();
        let mut vade_didcomm_listener = get_vade_transport("receiver", Some(channel.to_owned()))?;

        // start listener
        let mut receiver = vade_didcomm_listener.listen().await?;

        // send with second instance
        let vade_didcomm_sender = get_vade_transport("sender", Some(channel.to_owned()))?;
        let payload = ProtocolPayload {
            protocol: String::from("example"),
            step: String::from("example"),
        };
        let message = Message::new(payload, None as Option<()>).map_err(|e| e.to_string())?;
        vade_didcomm_sender.send_message(&message).await?;

        sleep(Duration::from_millis(5_000u64)).await;

        // fetch from receiver and extract payload
        let received_message = receiver.try_next()?.ok_or("no message received")?;
        let received_payload = serde_json::from_str::<ProtocolPayload>(&received_message.payload)?;
        let serialized_payload = serde_json::to_string(&received_payload)?;

        assert_eq!(&message.payload, &serialized_payload);

        Ok(())
    }
}
