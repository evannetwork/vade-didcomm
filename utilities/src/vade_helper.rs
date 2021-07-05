use vade::Vade;
use vade_didcomm::{AsyncResult, Message, VadeDidComm};
use serde::{Deserialize, Serialize};

pub async fn get_vade() -> AsyncResult<Vade> {
    let mut vade = Vade::new();
    let vade_didcomm = get_vade_didcomm().await?;
    vade.register_plugin(Box::from(vade_didcomm));

    Ok(vade)
}

pub async fn get_vade_didcomm() -> AsyncResult<VadeDidComm> {
    let vade_didcomm = VadeDidComm::new().await?;

    Ok(vade_didcomm)
}
