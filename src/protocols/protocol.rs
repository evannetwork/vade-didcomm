use crate::Message;

pub trait Protocol {
    fn beforeSend(message: Message) {
        panic!("afterSend not implemented")
    }

    fn afterSend() {
        panic!("afterSend not implemented")
    }
}
