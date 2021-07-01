use serde::{Deserialize, Serialize};
use serde_json::{Number};
use uuid::Uuid;
use didcomm_rs::{
    crypto::{CryptoAlgorithm},
    Message as DIDCommMessage,
};

pub struct MessageHandler {

}


impl MessageHandler {
    pub fn from_string(payload: &str) -> Message {
        let message: Message = serde_json::from_str(payload).unwrap();
        return message;
    }

    fn generate_id() -> String {
        Uuid::new_v4().to_simple().to_string()
    }

    pub fn encrypt(self) {
        let mut didcomm_message = DIDCommMessage::new()
            .body(serde_json::to_string(&self).as_bytes())
            .as_jwe(&CryptoAlgorithm::XC20P);
    }
}
