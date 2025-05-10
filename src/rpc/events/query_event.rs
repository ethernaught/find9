use std::time::{SystemTime, UNIX_EPOCH};
use rlibdns::messages::message_base::MessageBase;
use rlibdns::utils::dns_query::DnsQuery;
use crate::rpc::events::inter::event::Event;
use crate::rpc::events::inter::dns_message_event::DnsMessageEvent;

pub struct QueryEvent {
    prevent_default: bool,
    query: DnsQuery,
    received_time: u128,
    response: Option<MessageBase>
}

impl QueryEvent {

    pub fn new(query: DnsQuery) -> Self {
        Self {
            prevent_default: false,
            query,
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

impl Event for QueryEvent {

    fn is_prevent_default(&self) -> bool {
        self.prevent_default
    }

    fn prevent_default(&mut self) {
        self.prevent_default = true;
    }
}

impl DnsMessageEvent for QueryEvent {

    fn get_query(&self) -> &MessageBase {
        &self.query
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
