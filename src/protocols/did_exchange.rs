use uuid::Uuid;
use k256::elliptic_curve::rand_core::OsRng;

pub const DID_EXCHANGE_PROTOCOL_URL: &str = "https://didcomm.org/didexchange/1.0";

pub fn get_com_did_obj(
    from_did: &String,
    public_key_encoded: &String,
    service_endpoint: &String,
) -> String {
    return format!(
        r#"{{
            "@context": "https://w3id.org/did/v1",
            "id": "{0}",
            "publicKey": [{{
                "id": "{0}#key-1",
                "type": [
                  "Ed25519VerificationKey2018"
                ],
                "publicKeyBase58": "{1}"
            }}],
            "authentication": [
              "{0}#key-1"
            ],
            "service": [{{
                "id": "{0}#didcomm",
                "type": "did-communication",
                "priority": 0,
                "serviceEndpoint": "{2}",
                "recipientKeys": ["{1}"]
            }}]
          }}"#,
          from_did,
          public_key_encoded,
          service_endpoint,
    );
}

pub fn get_request_message(
    from_did: &String,
    to_did: &String,
    from_service_endpoint: &String,
) -> String {
    let keypair: ed25519_dalek::Keypair = ed25519_dalek::Keypair::generate(&mut OsRng);
    let encoded_pub_key = &hex::encode(keypair.public.to_bytes());
    let message_payload = get_com_did_obj(from_did, encoded_pub_key, from_service_endpoint);
    let thread_id = Uuid::new_v4().to_simple().to_string();
    let service_id = format!("{0}#key-1", from_did);
    let exchange_request =  format!(
        r#"{{
            "id": "{0}",
            "thid": "{1}",
            "pthid": "{0}#key-1",
            "type": "{5}/request",
            "from": "{2}",
            "to": ["{3}"],
            "body": {4},
        }}"#,
        thread_id,
        service_id,
        from_did,
        to_did,
        message_payload,
        DID_EXCHANGE_PROTOCOL_URL,
    );
    return exchange_request;
}
