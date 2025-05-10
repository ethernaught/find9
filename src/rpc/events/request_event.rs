use std::time::{SystemTime, UNIX_EPOCH};
use rlibdns::messages::message_base::MessageBase;
use crate::rpc::events::inter::event::Event;
use crate::rpc::events::inter::dns_message_event::DnsMessageEvent;

pub struct RequestEvent {
    prevent_default: bool,
    message: MessageBase,
    received_time: u128,
    response: Option<MessageBase>
}

impl RequestEvent {

    pub fn new(message: MessageBase) -> Self {
        Self {
            prevent_default: false,
            message,
            received_time: 0,
            response: None
        }
    }

    pub fn has_response(&self) -> bool {
        self.response.is_some()
    }

    pub fn get_response(&mut self) -> Option<&mut MessageBase> {
        self.response.as_mut()
    }

    pub fn set_response(&mut self, message: MessageBase) {
        self.response = Some(message);
    }
}

impl Event for RequestEvent {

    fn is_prevent_default(&self) -> bool {
        self.prevent_default
    }

    fn prevent_default(&mut self) {
        self.prevent_default = true;
    }
}

impl DnsMessageEvent for RequestEvent {

    fn get_message(&self) -> &MessageBase {
        &self.message
    }

    fn set_received_time(&mut self, received_time: u128) {
        self.received_time = received_time;
    }

    fn get_received_time(&self) -> u128 {
        self.received_time
    }

    fn received(&mut self) {
        self.received_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
    }
}
