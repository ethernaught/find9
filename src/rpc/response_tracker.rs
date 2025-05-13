use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::rpc::call::Call;

pub const MAX_ACTIVE_CALLS: usize = 512;
pub const STALLED_TIME: u128 = 60000;

#[derive(Clone)]
pub struct ResponseTracker {
    calls: Arc<Mutex<HashMap<u16, Call>>>,
}

impl ResponseTracker {

    pub fn new() -> Self {
        Self {
            calls: Arc::new(Mutex::new(HashMap::with_capacity(MAX_ACTIVE_CALLS))),
        }
    }

    pub fn add(&self, id: u16, call: Call) {
        self.calls.lock().unwrap().insert(id, call);
    }

    pub fn get(&self, id: u16) -> Option<Call> {
        self.calls.lock().unwrap().get(&id).cloned()
    }

    pub fn contains(&self, id: u16) -> bool {
        self.calls.lock().unwrap().contains_key(&id)
    }

    pub fn remove(&self, id: u16) -> Option<Call> {
        self.calls.lock().unwrap().remove(&id)
    }

    pub fn poll(&self, id: u16) -> Option<Call> {
        self.calls.lock().unwrap().remove(&id)
    }

    pub fn remove_stalled(&self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();

        let mut stalled = Vec::new();

        for (&id, call) in self.calls.lock().unwrap().iter() {
            if !call.is_stalled(now) {
                break;
            }

            stalled.push(id);
        }

        for id in stalled {
            self.calls.lock().unwrap().remove(&id);
        }
    }
}
