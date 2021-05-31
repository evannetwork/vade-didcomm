use crate::utils::AsyncResult;
use async_trait::async_trait;
use futures::channel::mpsc;
use serde::{Deserialize, Serialize};
use std::error::Error;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProtocolPayload {
    pub protocol: String,
    pub step: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: String,
    pub metadata: Option<String>,
    pub payload: String,
}

impl Message {
    #[allow(dead_code)]
    pub fn new<T1, T2>(
        payload: T1,
        metadata: Option<T2>,
    ) -> Result<Message, Box<dyn Error + Send + Sync>>
    where
        T1: Serialize,
        T2: Serialize,
    {
        let options_string = match metadata {
            Some(value) => Some(
                serde_json::to_string(&value)
                    .map_err(|err| format!("could not serialize options; {}", &err))?,
            ),
            None => None,
        };
        Ok(Message::new_from_string(
            serde_json::to_string(&payload)
                .map_err(|err| format!("could not serialize payload; {}", &err))?,
            options_string,
        ))
    }

    #[allow(dead_code)]
    pub fn new_from_string(payload: String, metadata: Option<String>) -> Message {
        Message {
            id: Message::generate_id(),
            metadata,
            payload,
        }
    }

    fn generate_id() -> String {
        Uuid::new_v4().to_simple().to_string()
    }
}

#[async_trait]
pub trait VadeTransport {
    async fn handle_message(&mut self, message_obj: Message) -> AsyncResult<()>;
    async fn listen(&mut self) -> AsyncResult<mpsc::UnboundedReceiver<Message>>;
    async fn send_message(&self, message: &Message) -> AsyncResult<()>;
}
