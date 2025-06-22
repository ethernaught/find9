use std::collections::HashMap;
use std::io;
use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::zone::zone_parser::ZoneParser;
use crate::dns::listeners::a_query::on_a_query;
use crate::dns::listeners::aaaa_query::on_aaaa_query;
use crate::dns::listeners::cname_query::on_cname_query;
use crate::dns::listeners::https_query::on_https_query;
use crate::dns::listeners::mx_query::on_mx_query;
use crate::dns::listeners::ns_query::on_ns_query;
use crate::dns::listeners::ptr_query::on_ptr_query;
use crate::dns::listeners::soa_query::on_soa_query;
use crate::dns::listeners::srv_query::on_srv_query;
use crate::dns::listeners::txt_query::on_txt_query;
use crate::dns::tcp_server::TcpServer;
use crate::dns::udp_server::UdpServer;
use crate::rpc::events::query_event::QueryEvent;
use crate::zone::inter::zone_types::ZoneTypes;
use crate::zone::zone::Zone;

pub type QueryMap = Arc<RwLock<HashMap<RRTypes, Box<dyn Fn(&mut QueryEvent) -> ResponseResult<()> + Send + Sync>>>>;
pub type ResponseResult<T> = Result<T, ResponseCodes>;

pub struct Dns {
    zones: Arc<RwLock<Zone>>,
    udp: UdpServer,
    tcp: TcpServer
}

impl Dns {

    pub fn new() -> Self {
        let zones = Arc::new(RwLock::new(Zone::new(ZoneTypes::Hint)));

        let udp = UdpServer::new();
        udp.register_query_listener(RRTypes::A, on_a_query(&zones));
        udp.register_query_listener(RRTypes::Aaaa, on_aaaa_query(&zones));
        udp.register_query_listener(RRTypes::Ns, on_ns_query(&zones));
        udp.register_query_listener(RRTypes::Txt, on_txt_query(&zones));
        udp.register_query_listener(RRTypes::Mx, on_mx_query(&zones)); //TEST
        udp.register_query_listener(RRTypes::Ptr, on_ptr_query(&zones));
        udp.register_query_listener(RRTypes::CName, on_cname_query(&zones));
        udp.register_query_listener(RRTypes::Srv, on_srv_query(&zones));
        udp.register_query_listener(RRTypes::Https, on_https_query(&zones));
        
        udp.register_query_listener(RRTypes::Soa, on_soa_query(&zones));

        let tcp = TcpServer::new();
        tcp.register_query_listener(RRTypes::A, on_a_query(&zones));
        tcp.register_query_listener(RRTypes::Aaaa, on_aaaa_query(&zones));
        tcp.register_query_listener(RRTypes::Ns, on_ns_query(&zones));
        tcp.register_query_listener(RRTypes::Txt, on_txt_query(&zones));
        tcp.register_query_listener(RRTypes::Mx, on_mx_query(&zones)); //TEST
        tcp.register_query_listener(RRTypes::Ptr, on_ptr_query(&zones));
        tcp.register_query_listener(RRTypes::CName, on_cname_query(&zones));
        tcp.register_query_listener(RRTypes::Srv, on_srv_query(&zones));
        tcp.register_query_listener(RRTypes::Https, on_https_query(&zones));

        tcp.register_query_listener(RRTypes::Soa, on_soa_query(&zones));

        Self {
            zones,
            udp,
            tcp
        }
    }

    pub fn start(&mut self, port: u16) -> io::Result<()> {
        self.udp.run(port)?;
        self.tcp.run(port)?;
        Ok(())
    }

    pub fn is_running(&self) -> (bool, bool) {
        (self.udp.is_running(), self.tcp.is_running())
    }

    pub fn stop(&self) {
        self.udp.kill();
    }

    pub fn get_udp(&self) -> &UdpServer {
        &self.udp
    }

    pub fn get_tcp(&self) -> &TcpServer {
        &self.tcp
    }

    pub fn register_zone(&self, file_path: &str, domain: &str) -> io::Result<()> {
        let mut zone = Zone::new(ZoneTypes::Master);

        let mut parser = ZoneParser::new(file_path, domain)?;
        for (name, record) in parser.iter() {
            match name.as_str() {
                "." => self.zones.write().unwrap().add_record(record), //BE CAREFUL WITH THIS ONE - DONT ALLOW MOST OF THE TIME
                "@" => zone.add_record(record),
                //_ => zone.add_record_to(&name, record, ZoneTypes::Master)
                _ => {
                    if name == "c5" {
                        zone.add_record_to(&name, record, ZoneTypes::Hint)
                    } else {
                        zone.add_record_to(&name, record, ZoneTypes::Master)
                    }
                }
            }
        }

        self.zones.write().unwrap().add_zone_to(&parser.get_origin(), zone, ZoneTypes::Hint);

        println!("{:?}", self.zones.read().unwrap());
        Ok(())
    }
}
