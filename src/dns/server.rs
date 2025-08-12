use std::io;
use std::thread::JoinHandle;
use rlibdns::messages::inter::rr_types::RRTypes;
use crate::dns::dns::ResponseResult;
use crate::rpc::events::query_event::QueryEvent;

pub trait Server {

    fn run(&mut self, port: u16) -> io::Result<JoinHandle<()>>;

    fn is_running(&self) -> bool;

    fn kill(&self);

    fn register_query_listener<F>(&self, key: RRTypes, callback: F)
    where
        F: Fn(&mut QueryEvent) -> ResponseResult<()> + Send + Sync + 'static;
}
