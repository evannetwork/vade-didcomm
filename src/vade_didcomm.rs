use crate::{
    message::{Message},
    AsyncResult,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use vade::{VadePlugin, VadePluginResultValue};

big_array! { BigArray; }

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidcommSendOptions {
    pub encryption_key: [u8; 32],
    #[serde(with = "BigArray")]
    pub sign_keypair: [u8; 64],
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidcommReceiveOptions {
    pub decryption_key: [u8; 32],
    pub sign_public: [u8; 32],
}

#[allow(dead_code)]
pub struct VadeDidComm { }

impl VadeDidComm {
    /// Creates new instance of `VadeDidComm`.
    pub async fn new() -> AsyncResult<VadeDidComm> {
        match env_logger::try_init() {
            Ok(_) | Err(_) => (),
        };
        let vade_didcomm = VadeDidComm { };

        Ok(vade_didcomm)
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
        let encrypted = Message::encrypt(
            &payload,
            &options.encryption_key,
            &options.sign_keypair,
        )?;

        Ok(VadePluginResultValue::Success(encrypted))
    }

    async fn didcomm_receive(
        &mut self,
        options: &str,
        payload: &str,
    ) -> AsyncResult<VadePluginResultValue<Option<String>>> {
        log::debug!("handling receival of DIDComm message");

        let options = serde_json::from_str::<DidcommReceiveOptions>(&options)?;

        let decrypted = Message::decrypt(
            &payload,
            &options.decryption_key,
            &options.sign_public,
        )?;

        Ok(VadePluginResultValue::Success(Some(serde_json::to_string(
            &decrypted.body,
        )?)))
    }
}
