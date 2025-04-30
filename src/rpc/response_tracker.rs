use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::rpc::call::Call;

pub const MAX_ACTIVE_CALLS: usize = 512;
pub const STALLED_TIME: u128 = 60000;

pub struct ResponseTracker {
    calls: HashMap<u16, Call>,
}

impl ResponseTracker {

    pub fn new() -> Self {
        Self {
            calls: HashMap::with_capacity(MAX_ACTIVE_CALLS),
        }
    }

    pub fn add(&mut self, id: u16, call: Call) {
        self.calls.insert(id, call);
    }

    pub fn get(&self, id: u16) -> Option<&Call> {
        self.calls.get(&id)
    }

    pub fn contains(&self, id: u16) -> bool {
        self.calls.contains_key(&id)
    }

    pub fn remove(&mut self, id: u16) -> Option<Call> {
        self.calls.remove(&id)
    }

    pub fn poll(&mut self, id: u16) -> Option<Call> {
        self.calls.remove(&id)
    }

    pub fn remove_stalled(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();

        let mut stalled = Vec::new();

        for (&id, call) in self.calls.iter() {
            if !call.is_stalled(now) {
                break;
            }

            stalled.push(id);
        }

        for id in stalled {
            self.calls.remove(&id);
        }
    }
}
