use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::rpc::response_tracker::STALLED_TIME;

#[derive(Copy, Clone)]
pub struct Call {
    address: SocketAddr,
    sent_time: u128
}

impl Call {

    pub fn new(address: SocketAddr) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();

        Self {
            address,
            sent_time: now
        }
    }

    pub fn get_address(&self) -> &SocketAddr {
        &self.address
    }

    pub fn set_sent_time(&mut self, sent_time: u128) {
        self.sent_time = sent_time;
    }

    pub fn get_sent_time(&self) -> u128 {
        self.sent_time
    }

    pub fn is_stalled(&self, now: u128) -> bool {
        now-self.sent_time > STALLED_TIME
    }
}
