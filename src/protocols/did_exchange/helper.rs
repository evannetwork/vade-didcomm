use std::collections::HashMap;

use data_encoding::BASE64;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    datatypes::{
        Base64Container,
        BaseMessage,
        CommunicationDidDocument,
        DidCommOptions,
        DidCommPubKey,
        DidCommService,
        DidDocumentBodyAttachment,
        ExchangeInfo,
        MessageWithBody,
    },
    protocols::did_exchange::DID_EXCHANGE_PROTOCOL_URL,
    utils::hex_option,
};

/// Specifies all possible message directions.
#[derive(PartialEq)]
pub enum DidExchangeType {
    Request,
    Response,
}

/// Object with base64 encoded value
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DidExchangeOptions {
    pub service_endpoint: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "hex_option")]
    pub did_exchange_my_secret: Option<[u8; 32]>,
    #[serde(flatten)]
    pub didcomm_options: DidCommOptions,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DidExchangeBaseMessage {
    #[serde(flatten)]
    pub base_message: BaseMessage,
    pub id: Option<String>,
    pub pthid: Option<String>,
    pub thid: Option<String>,
}

/// Creates a new communication DID document for a specific DID, a communication pub key and the
/// service url, where the user can be reached.
///
/// # Arguments
/// * `from_did` - DID to build the DID document for
/// * `public_key_encoded` - communication pub key for the DID exchange that will be sent to the target
/// * `service_endpoint` - url where the user can be reached
///
/// # Returns
/// * `CommunicationDidDocument` - constructed DIDComm object, ready to be sent
pub fn get_communication_did_doc(
    from_did: &str,
    public_key_encoded: &str,
    service_endpoint: &str,
) -> CommunicationDidDocument {
    let key_id = format!("{}#key-1", from_did);
    let pub_key_vec = vec![DidCommPubKey {
        id: key_id.to_owned(),
        r#type: [String::from("Ed25519VerificationKey2018")].to_vec(),
        public_key_base_58: public_key_encoded.to_string(),
    }];

    let service_vec = vec![DidCommService {
        id: format!("{}#didcomm", from_did),
        r#type: String::from("did-communication"),
        priority: 0,
        service_endpoint: service_endpoint.to_string(),
        recipient_keys: [public_key_encoded.to_string()].to_vec(),
    }];

    CommunicationDidDocument {
        context: String::from("https://w3id.org/did/v1"),
        id: from_did.to_string(),
        public_key: pub_key_vec,
        authentication: vec![key_id],
        service: service_vec,
    }
}

/// Constructs a new DID exchange message, including the DIDComm object as message body.
///
/// # Arguments
/// * `step_type` - step to build the message type (request, response)
/// * `from_did` - DID that sends the message
/// * `to_did` - DID that receives the message
/// * `from_service_endpoint` - url where the user can be reached
/// * `encoded_keypair` - communication keypair (only pubkey will be used)
///
/// # Returns
/// * `MessageWithBody<CommunicationDidDocument>` - constructed DIDComm object, ready to be sent
#[allow(clippy::type_complexity)]
pub fn get_did_exchange_message(
    step_type: DidExchangeType,
    from_did: &str,
    key_agreement_did: &str,
    to_did: &str,
    from_service_endpoint: &str,
    pub_key: &str,
    message: &DidExchangeBaseMessage,
) -> Result<
    (
        MessageWithBody<DidDocumentBodyAttachment<Base64Container>>,
        CommunicationDidDocument,
    ),
    Box<dyn std::error::Error>,
> {
    let message = message.clone();
    // convert this to doc attach with base 64 use data_encoding::BASE64;
    let did_document = get_communication_did_doc(key_agreement_did, pub_key, from_service_endpoint);
    let base64_encoded_did_document =
        BASE64.encode(serde_json::to_string(&did_document)?.as_bytes());
    let fallback_id = Uuid::new_v4().to_simple().to_string();
    let service_id = format!("{0}#key-1", key_agreement_did);
    let step_name = match step_type {
        DidExchangeType::Request => "request",
        DidExchangeType::Response => "response",
    };
    let exchange_request: MessageWithBody<DidDocumentBodyAttachment<Base64Container>> =
        MessageWithBody {
            body: Some(DidDocumentBodyAttachment {
                did_doc_attach: Base64Container {
                    base64: base64_encoded_did_document,
                },
                label: match message.base_message.body.get("label") {
                    Some(label) => serde_json::from_value(label.clone())?,
                    None => None,
                },
            }),
            created_time: None,
            expires_time: None,
            from: Some(String::from(from_did)),
            id: message
                .id
                .as_ref()
                .or(Some(&fallback_id))
                .map(|v| v.to_owned()),
            other: HashMap::new(),
            pthid: message
                .pthid
                .or_else(|| Some(format!("{}#key-1", fallback_id))),
            r#type: format!("{}/{}", DID_EXCHANGE_PROTOCOL_URL, step_name),
            thid: message.thid.or(Some(service_id)),
            to: Some([String::from(to_did)].to_vec()),
        };

    Ok((exchange_request, did_document))
}

/// Takes an DIDComm message and extracts all necessary information to process it during request /
/// response.
///
/// # Arguments
/// * `message` - DIDComm message with communication DID document as body
///
/// # Returns
/// * `ExchangeInfo` - necessary information
pub fn get_exchange_info_from_message(
    message: &BaseMessage,
    did_document: CommunicationDidDocument,
) -> Result<ExchangeInfo, Box<dyn std::error::Error>> {
    let message = message.clone();
    let from_did = message.from.ok_or("from is required")?;

    let to_vec = message.to.ok_or("to is required")?;
    if to_vec.is_empty() {
        return Err(Box::from(
            "DID exchange requires at least one DID in the to field.",
        ));
    }
    let to_did = &to_vec[0];
    if did_document.public_key.is_empty() {
        return Err(Box::from(
            "No pub key was attached to the communication DID document.",
        ));
    }
    let pub_key_base58 = &did_document.public_key[0].public_key_base_58;
    let pub_key_bytes = bs58::decode(pub_key_base58).into_vec()?;
    let pub_key_hex = hex::encode(pub_key_bytes);
    if did_document.service.is_empty() {
        return Err(Box::from(
            "No service_endpoint was attached to the communication DID document.",
        ));
    }
    let service_endpoint = &did_document.service[0].service_endpoint;

    Ok(ExchangeInfo {
        from: from_did,
        to: String::from(to_did),
        did_id: did_document.id,
        pub_key_hex,
        service_endpoint: String::from(service_endpoint),
    })
}

pub fn get_did_document_from_body(
    message: &str,
) -> Result<CommunicationDidDocument, Box<dyn std::error::Error>> {
    let message_with_base64_did_document: MessageWithBody<
        DidDocumentBodyAttachment<Base64Container>,
    > = serde_json::from_str(message)?;
    let did_document_base64_encoded_string = message_with_base64_did_document
        .body
        .ok_or("body is a required field for DID exchange messages")?
        .did_doc_attach
        .base64;
    let did_document_base64_encoded_bytes = did_document_base64_encoded_string.as_bytes();
    let did_document_bytes = BASE64.decode(did_document_base64_encoded_bytes)?;
    let did_document_string = std::str::from_utf8(&did_document_bytes)?;
    let did_document: CommunicationDidDocument = serde_json::from_str(did_document_string)?;
    Ok(did_document)
}
