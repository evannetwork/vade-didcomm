use utilities::keypair::{KeyPairSet, get_keypair_set};
use vade::Vade;
use vade_didcomm::{AsyncResult, Message, VadeDidComm, get_request_message};
use serde::{Deserialize, Serialize};
use k256::elliptic_curve::rand_core::OsRng;
use x25519_dalek::{PublicKey, StaticSecret};

async fn get_vade() -> AsyncResult<Vade> {
    let mut vade = Vade::new();
    let vade_didcomm = VadeDidComm::new().await?;
    vade.register_plugin(Box::from(vade_didcomm));

    Ok(vade)
}

async fn do_request(
    inviter: &String,
    invitee: &String,
) {
    let exchange_request = get_request_message(
        &inviter,
        &invitee,
        &String::from("http://evan.network"),
    );
    // let results = vade.didcomm_send(&options, &exchange_request).await?;

    // panic!("{}", exchange_request);

}

#[tokio::test]
async fn can_do_key_exchange() -> AsyncResult<()> {
    let mut vade = get_vade().await?;
    let sign_keypair = get_keypair_set();
    let inviter = String::from("did:uknow:d34db33d");
    let invitee = String::from("did:uknow:d34db33f");

    do_request(&inviter, &invitee).await;

    Ok(())
}
