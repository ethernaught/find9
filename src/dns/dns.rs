use std::collections::HashMap;
use std::io;
use std::io::ErrorKind;
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::zone::zone_parser::ZoneParser;
use crate::dns::listeners::a_query::on_a_query;
use crate::dns::listeners::aaaa_query::on_aaaa_query;
use crate::dns::listeners::ns_query::on_ns_query;
use crate::dns::listeners::soa_query::on_soa_query;
use crate::dns::listeners::txt_query::on_txt_query;
use crate::dns::tcp_server::TcpServer;
use crate::dns::udp_server;
use crate::dns::udp_server::UdpServer;
use crate::rpc::events::query_event::QueryEvent;
use crate::zone::inter::zone_types::ZoneTypes;
use crate::zone::zone::Zone;

pub type QueryMap = Arc<RwLock<HashMap<RRTypes, Vec<Box<dyn Fn(&mut QueryEvent) -> io::Result<()> + Send + Sync>>>>>;

pub struct Dns {
    zones: Arc<RwLock<HashMap<String, Zone>>>,
    udp: UdpServer,
    tcp: TcpServer
}

impl Dns {

    pub fn new() -> Self {
        let zones = Arc::new(RwLock::new(HashMap::new()));

        let udp = UdpServer::new();
        udp.register_query_listener(RRTypes::A, on_a_query(&zones));
        //_self.register_query_listener(RRTypes::Aaaa, on_aaaa_query());
        //_self.register_query_listener(RRTypes::Ns, on_ns_query());
        //_self.register_query_listener(RRTypes::Txt, on_txt_query());
        //_self.register_query_listener(RRTypes::Soa, on_soa_query());

        let tcp = TcpServer::new();
        
        Self {
            zones,
            udp,
            tcp
        }
    }

    pub fn start(&mut self, port: u16) -> io::Result<()> {
        self.udp.run(port)?;
        Ok(())
    }

    pub fn is_running(&self) -> (bool, bool) {
        (self.udp.is_running(), false)
    }

    pub fn stop(&self) {
        self.udp.kill();
    }

    pub fn get_udp(&self) -> &UdpServer {
        &self.udp
    }

    pub fn register_zone(&self, file_path: &str, domain: &str) -> io::Result<()> {
        let mut zone = Zone::new(ZoneTypes::Master);

        let mut parser = ZoneParser::new(file_path, domain)?;
        for (name, record) in parser.iter() {
            if !name.eq(domain){
                continue;
            }

            zone.add_record(record);
        }

        self.zones.write().unwrap().insert(parser.get_origin(), zone);
        Ok(())
    }
}
