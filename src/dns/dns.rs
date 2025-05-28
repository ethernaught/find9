use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use rlibdns::messages::inter::rr_types::RRTypes;
use crate::dns::listeners::a_query::on_a_query;
use crate::dns::listeners::aaaa_query::on_aaaa_query;
use crate::dns::listeners::ns_query::on_ns_query;
use crate::dns::listeners::soa_query::on_soa_query;
use crate::dns::listeners::txt_query::on_txt_query;
use crate::dns::udp_server::UdpServer;
use crate::rpc::events::query_event::QueryEvent;
use crate::zone::zone::Zone;

pub type QueryMap = HashMap<RRTypes, Vec<Box<dyn Fn(&mut QueryEvent) -> io::Result<()> + Send>>>;

pub struct Dns {
    zones: Arc<Mutex<HashMap<String, Zone>>>,
    query_mapping: QueryMap,
    udp_server: Option<JoinHandle<()>>,
}

impl Dns {

    pub fn new() -> Self {
        let _self = Self {
            zones: Arc::new(Mutex::new(HashMap::new())),
            query_mapping: QueryMap::new(),
            udp_server: None
        };

        //BASED ON CONFIG ENABLE SPECIFIC QUERY LISTENERS
        _self.register_query_listener(RRTypes::A, on_a_query());
        _self.register_query_listener(RRTypes::Aaaa, on_aaaa_query());
        _self.register_query_listener(RRTypes::Ns, on_ns_query());
        _self.register_query_listener(RRTypes::Txt, on_txt_query());
        _self.register_query_listener(RRTypes::Soa, on_soa_query());

        //let udp_server = UdpServer::new();

        _self
    }

    pub fn start(&mut self, port: u16) -> io::Result<JoinHandle<()>> {
        self.server.start(port)
    }

    pub fn stop(&self) {
        self.server.stop()
    }

    pub fn get_server(&mut self) -> &mut Server {
        &mut self.server
    }

    pub fn register_query_listener<F>(&self, key: RRTypes, callback: F)
    where
        F: Fn(&mut QueryEvent) -> io::Result<()> + Send + 'static
    {
        if self.query_mapping.lock().unwrap().contains_key(&key) {
            self.query_mapping.lock().unwrap().get_mut(&key).unwrap().push(Box::new(callback));
            return;
        }
        self.query_mapping.lock().unwrap().insert(key, vec![Box::new(callback)]);
    }
}
