use crate::{
    message::{Message, ProtocolPayload},
    AsyncResult,
    ResultAsyncifier,
};

pub struct ProtocolHandler {}

impl ProtocolHandler {
    async fn handle_step_receive(
        protocol: String,
        step: String,
    ) -> AsyncResult<Message> {
        log::info!("handling step receive {}:{}", &protocol, &step);
        match protocol.as_str() {
            "pingpong" => match step.as_str() {
                "ping" => Self::ping_receive(String::from(""), String::from("")).await,
                "pong" => Self::ping_send(String::from(""), String::from("")).await,
                _ => {
                    return Err(Box::from(format!(
                        r#"step {} for DIDComm protocol "{}" not supported"#,
                        &step, &protocol,
                    )));
                }
            },
            _ => {
                return Err(Box::from(format!(
                    r#"DIDComm protocol "{}" not supported"#,
                    &protocol,
                )));
            }
        }
    }

    pub async fn ping_receive(
        _options: String,
        _payload: String,
    ) -> AsyncResult<Message> {
        log::debug!("ping_receive");
        // prepare message
        let message_payload = ProtocolPayload {
            protocol: String::from("pingpong"),
            step: String::from("pong"),
        };
        let message = Message::new(message_payload, None as Option<()>).asyncify()?;

        return Ok(message);
    }

    pub async fn ping_send(
        _options: String,
        _payload: String,
    ) -> AsyncResult<Message> {
        log::debug!("ping_send");
        // prepare message
        let message_payload = ProtocolPayload {
            protocol: String::from("pingpong"),
            step: String::from("ping"),
        };
        let message = Message::new(message_payload, None as Option<()>).asyncify()?;
        return Ok(message);
    }
}
