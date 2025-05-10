use rlibdns::utils::dns_query::DnsQuery;
use crate::rpc::events::inter::event::Event;

pub trait DnsQueryEvent: Event {

    fn get_query(&self) -> &DnsQuery;

    fn set_received_time(&mut self, received_time: u128);

    fn get_received_time(&self) -> u128;

    fn received(&mut self);
}
