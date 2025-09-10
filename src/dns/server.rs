use std::io;
use std::thread::JoinHandle;
use rlibdns::messages::inter::op_codes::OpCodes;
use rlibdns::messages::inter::rr_classes::RRClasses;
use rlibdns::messages::inter::rr_types::RRTypes;
use crate::dns::dns::ResponseResult;
use crate::rpc::events::request_event::RequestEvent;

pub trait Server {

    fn run(&mut self, port: u16) -> io::Result<JoinHandle<()>>;

    fn is_running(&self) -> bool;

    fn kill(&self);

    fn register_request_listener<F>(&self, op_code: OpCodes, class: RRClasses, _type: RRTypes, callback: F)
    where
        F: Fn(&mut RequestEvent) -> ResponseResult<()> + Send + Sync + 'static;
}
