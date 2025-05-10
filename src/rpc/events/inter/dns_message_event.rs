use rlibdns::messages::message_base::MessageBase;
use crate::rpc::events::inter::event::Event;

pub trait DnsMessageEvent: Event {

    fn get_message(&self) -> &MessageBase;

    fn set_received_time(&mut self, received_time: u128);

    fn get_received_time(&self) -> u128;

    fn received(&mut self);
}
