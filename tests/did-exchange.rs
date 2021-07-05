use utilities::keypair::get_signed_keypair;
use vade::Vade;
use vade_didcomm::{AsyncResult, Message, VadeDidComm};
use serde::{Deserialize, Serialize};
use k256::elliptic_curve::rand_core::OsRng;
use x25519_dalek::{PublicKey, StaticSecret};

async fn get_vade() -> AsyncResult<Vade> {
    let mut vade = Vade::new();
    let vade_didcomm = VadeDidComm::new().await?;
    vade.register_plugin(Box::from(vade_didcomm));

    Ok(vade)
}

fn get_com_did_obj(
    invitee: String,
    public_key_encoded: String,
    service_endpoint: String,
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
          invitee,
          public_key_encoded,
          service_endpoint,
    );
}

#[tokio::test]
async fn can_do_key_exchange() -> AsyncResult<()> {
    let mut vade = get_vade().await?;
    let inviter_secret = StaticSecret::new(OsRng);
    let inviter_public = PublicKey::from(&inviter_secret);
    let invitee_secret = StaticSecret::new(OsRng);
    let invitee_public = PublicKey::from(&invitee_secret);
    let sign_keypair: ed25519_dalek::Keypair = ed25519_dalek::Keypair::generate(&mut OsRng);
    let inviter = String::from("did:uknow:d34db33d");
    let invitee = String::from("did:uknow:d34db33f");

    let didcomm_obj = get_com_did_obj(
        invitee,
        String::from_utf8(inviter_public.as_bytes().iter().cloned().collect())?,
        String::from("http://evan.network"),
    );

    println!("{}", &didcomm_obj);

    Ok(())
}
