use crate::{
    vade_transport::{Message, ProtocolPayload, VadeTransport},
    AsyncResult,
    ResultAsyncifier,
};
use async_trait::async_trait;
use didcomm_rs::{
    crypto::{CryptoAlgorithm, SignatureAlgorithm},
    Message as DIDCommMessage,
};
use futures::lock::Mutex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::time::sleep;
use vade::{VadePlugin, VadePluginResultValue};

const TRANSFER_DIDCOMM: &str = "didcomm";

big_array! { BigArray; }

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferOptions {
    pub transfer: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidcommSendOptions {
    pub encryption_key: [u8; 32],
    #[serde(with = "BigArray")]
    pub sign_keypair: [u8; 64],
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DidcommSendPayload {
    pub body: String,
    pub from: Option<String>,
    pub kid: Option<String>,
    pub to: Option<Vec<String>>,
    #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
    pub(crate) other: HashMap<String, String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidcommReceiveOptions {
    pub decryption_key: [u8; 32],
    pub sign_public: [u8; 32],
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidcommReceiveResult {
    pub message: DIDCommMessage,
    pub body: String,
}

macro_rules! parse {
    ($data:expr, $type_name:expr) => {{
        serde_json::from_str($data)
            .map_err(|e| format!("{} when parsing {} {}", &e, $type_name, $data))?
    }};
}

macro_rules! apply_optional {
    ($message:ident, $payload:ident, $payload_arg:ident) => {{
        match $payload.$payload_arg {
            Some(value) => {
                $message = $message.$payload_arg(&value);
            }
            _ => (),
        }
    }};
}

macro_rules! ignore_unrelated {
    ($method:expr, $options:expr) => {{
        let type_options: TransferOptions = parse!($options, "options");
        match type_options.transfer.as_deref() {
            Some(TRANSFER_DIDCOMM) => (),
            _ => return Ok(VadePluginResultValue::Ignored),
        };
    }};
}

struct ProtocolHandler {}

impl ProtocolHandler {
    async fn handle_step_receive(
        transport: Arc<Mutex<Box<dyn VadeTransport + Send + Sync>>>,
        protocol: String,
        step: String,
    ) -> AsyncResult<Option<String>> {
        log::info!("handling step receive {}:{}", &protocol, &step);
        match protocol.as_str() {
            "pingpong" => match step.as_str() {
                "ping" => Self::ping_receive(transport, String::from(""), String::from("")).await,
                "pong" => Self::ping_send(transport, String::from(""), String::from("")).await,
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

    async fn ping_receive(
        transport: Arc<Mutex<Box<dyn VadeTransport + Send + Sync>>>,
        _options: String,
        _payload: String,
    ) -> AsyncResult<Option<String>> {
        log::debug!("ping_receive");
        // prepare message
        let message_payload = ProtocolPayload {
            protocol: String::from("pingpong"),
            step: String::from("pong"),
        };
        let message = Message::new(message_payload, None as Option<()>).asyncify()?;

        // send it
        transport.lock().await.send_message(&message).await?;

        // protocol ends here as we responded

        Ok(None)
    }

    async fn ping_send(
        transport: Arc<Mutex<Box<dyn VadeTransport + Send + Sync>>>,
        _options: String,
        _payload: String,
    ) -> AsyncResult<Option<String>> {
        log::debug!("ping_send");
        // prepare message
        let message_payload = ProtocolPayload {
            protocol: String::from("pingpong"),
            step: String::from("ping"),
        };
        let message = Message::new(message_payload, None as Option<()>).asyncify()?;

        // send it
        transport.lock().await.send_message(&message).await?;

        // no need to wait here, as generic handler will manage received message

        Ok(None)
    }
}

#[allow(dead_code)]
pub struct VadeDidComm {
    signer: String,
    target: String,
    transport: Arc<Mutex<Box<dyn VadeTransport + Send + Sync>>>,
}

impl VadeDidComm {
    /// Creates new instance of `VadeDidComm`.
    pub async fn new(
        signer: String,
        target: String,
        transport: Box<dyn VadeTransport + Send + Sync>,
    ) -> AsyncResult<VadeDidComm> {
        match env_logger::try_init() {
            Ok(_) | Err(_) => (),
        };
        let vade_didcomm = VadeDidComm {
            signer,
            target,
            transport: Arc::new(Mutex::new(transport)),
        };

        Ok(vade_didcomm)
    }

    async fn handle_protocol_start(
        &mut self,
        _method: String,
        protocol: String,
        options: String,
        payload: String,
    ) -> AsyncResult<Option<String>> {
        match protocol.as_str() {
            "pingpong" => {
                ProtocolHandler::ping_send(self.transport.clone(), options, payload).await
            }
            "listen" => {
                self.listen().await?;
                Ok(None)
            }
            _ => {
                return Err(Box::from(format!(
                    r#"DIDComm protocol "{}" not supported"#,
                    &protocol,
                )));
            }
        }
    }

    async fn listen(&self) -> AsyncResult<()> {
        log::debug!("starting didcomm listener");

        let local_transport = self.transport.clone();
        let mut receiver = local_transport.lock().await.listen().await?;

        let get_message_in_loop = async move {
            let error_trap = async move {
                loop {
                    match receiver.try_next() {
                        Ok(Some(value)) => {
                            log::debug!("got message from receiver: {:?}", &value);
                            let payload: ProtocolPayload = serde_json::from_str(&value.payload)?;
                            ProtocolHandler::handle_step_receive(
                                local_transport.clone(),
                                payload.protocol,
                                payload.step,
                            )
                            .await?;
                        }
                        Ok(None) => {
                            log::debug!("channel disconnected, stop listening");
                            break;
                        }
                        Err(_) => {
                            // no message received, try again
                            sleep(Duration::from_millis(10u64)).await;
                        }
                    };
                }
                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            };
            match error_trap.await {
                Ok(_) => {
                    log::info!("listener gracefully shut down")
                }
                Err(err) => {
                    log::warn!("listener ran into an error: {}", &err)
                }
            };
        };

        tokio::task::spawn(get_message_in_loop);

        Ok(())
    }

    fn receive_message(
        &mut self,
        message: &str,
        decryption_key: Option<&[u8]>,
        validation_key: Option<&[u8]>,
    ) -> AsyncResult<DidcommReceiveResult> {
        log::debug!("receiving message");

        let received =
            DIDCommMessage::receive(&message, decryption_key, validation_key).map_err(|err| {
                format!(
                    "could not get valid message from received data: {}",
                    &err.to_string()
                )
            })?;

        let body = String::from_utf8(received.body.clone())?;

        // will later on handle received messages internally as well

        Ok(DidcommReceiveResult {
            message: received,
            body,
        })
    }
}

#[async_trait]
impl VadePlugin for VadeDidComm {
    async fn didcomm_send(
        &mut self,
        options: &str,
        payload: &str,
    ) -> AsyncResult<VadePluginResultValue<Option<String>>> {
        log::debug!("preparing DIDComm message for being sent");

        let options = serde_json::from_str::<DidcommSendOptions>(&options)?;
        let payload = serde_json::from_str::<DidcommSendPayload>(&payload)?;

        // create new message with basic setup
        let mut message = DIDCommMessage::new()
            .body(payload.body.as_bytes())
            .as_jwe(&CryptoAlgorithm::XC20P);

        match payload.to {
            Some(values) => {
                let to: Vec<&str> = values.iter().map(AsRef::as_ref).collect();
                message = message.to(&to);
            }
            _ => (),
        };

        // apply optional headers to known sections, use remaining as custom headers
        apply_optional!(message, payload, from);
        apply_optional!(message, payload, kid);

        // insert custom headers
        for (key, val) in payload.other.iter() {
            message = message.add_header_field(key.to_owned(), val.to_owned());
        }

        // finally sign and encrypt
        let ready_to_send = message
            .seal_signed(
                &options.encryption_key,
                &options.sign_keypair,
                SignatureAlgorithm::EdDsa,
            )
            .unwrap();

        Ok(VadePluginResultValue::Success(Some(ready_to_send)))
    }

    async fn didcomm_receive(
        &mut self,
        options: &str,
        payload: &str,
    ) -> AsyncResult<VadePluginResultValue<Option<String>>> {
        log::debug!("handling receival of DIDComm message");

        let options = serde_json::from_str::<DidcommReceiveOptions>(&options)?;

        let decrypted = self.receive_message(
            &payload,
            Some(&options.decryption_key),
            Some(&options.sign_public),
        )?;

        Ok(VadePluginResultValue::Success(Some(serde_json::to_string(
            &decrypted,
        )?)))
    }

    async fn run_custom_function(
        &mut self,
        method: &str,
        function: &str,
        options: &str,
        payload: &str,
    ) -> AsyncResult<VadePluginResultValue<Option<String>>> {
        ignore_unrelated!(method, options);
        // TODO swo: check if we're actually handling a protocol here
        Ok(VadePluginResultValue::Success(
            self.handle_protocol_start(
                String::from(method),
                String::from(function),
                String::from(options),
                String::from(payload),
            )
            .await
            .map_err(|err| err.to_string())?,
        ))
    }
}
