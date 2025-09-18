use std::collections::HashMap;
use std::io;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, RwLock};
use rlibdns::journal::inter::txn_op_codes::TxnOpCodes;
use rlibdns::journal::journal::Journal;
use rlibdns::messages::inter::op_codes::OpCodes;
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::inter::rr_classes::RRClasses;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::a_record::ARecord;
use rlibdns::records::inter::record_base::RecordBase;
use rlibdns::records::soa_record::SoaRecord;
use rlibdns::zone::inter::zone_types::ZoneTypes;
use rlibdns::zone::zone::Zone;
use rlibdns::zone::zone_store::ZoneStore;
use crate::dns::listeners::a_query::on_a_query;
use crate::dns::listeners::aaaa_query::on_aaaa_query;/*
use crate::dns::listeners::any_query::on_any_query;
use crate::dns::listeners::axfr_query::on_axfr_query;
use crate::dns::listeners::cname_query::on_cname_query;
use crate::dns::listeners::https_query::on_https_query;
use crate::dns::listeners::ixfr_query::on_ixfr_query;
use crate::dns::listeners::loc_query::on_uri_query;
use crate::dns::listeners::mx_query::on_mx_query;
use crate::dns::listeners::naptr_query::on_naptr_query;
use crate::dns::listeners::ns_query::on_ns_query;
use crate::dns::listeners::ptr_query::on_ptr_query;
use crate::dns::listeners::smimea_query::on_smimea_query;
use crate::dns::listeners::soa_query::on_soa_query;
use crate::dns::listeners::srv_query::on_srv_query;
use crate::dns::listeners::sshfp_query::on_sshfp_query;
use crate::dns::listeners::svcb_query::on_svcb_query;
use crate::dns::listeners::txt_query::on_txt_query;
use crate::dns::listeners::uri_query::on_loc_query;*/
use crate::dns::server::Server;
use crate::dns::tcp_server::TcpServer;
use crate::dns::udp_server::UdpServer;
use crate::rpc::events::request_event::RequestEvent;

pub type RequestMap = Arc<RwLock<HashMap<(OpCodes, RRTypes), Box<dyn Fn(&mut RequestEvent) -> ResponseResult<()> + Send + Sync>>>>;
pub type ResponseResult<T> = Result<T, ResponseCodes>;

pub struct Dns {
    store: Arc<RwLock<ZoneStore>>,
    udp: UdpServer,
    tcp: TcpServer
}

impl Dns {

    pub fn new() -> Self {
        let store = Arc::new(RwLock::new(ZoneStore::new()));

        let udp = UdpServer::new();
        udp.register_request_listener(OpCodes::Query, RRTypes::A, on_a_query(&store));
        udp.register_request_listener(OpCodes::Query, RRTypes::Aaaa, on_a_query(&store));/*
        udp.register_request_listener(OpCodes::Query, RRTypes::Ns, on_ns_query(&store));
        udp.register_request_listener(OpCodes::Query, RRTypes::Txt, on_txt_query(&store));
        udp.register_request_listener(OpCodes::Query, RRTypes::Mx, on_mx_query(&store)); //TEST
        udp.register_request_listener(OpCodes::Query, RRTypes::Ptr, on_ptr_query(&store));
        udp.register_request_listener(OpCodes::Query, RRTypes::CName, on_cname_query(&store));
        udp.register_request_listener(OpCodes::Query, RRTypes::Srv, on_srv_query(&store));
        udp.register_request_listener(OpCodes::Query, RRTypes::Naptr, on_naptr_query(&store));
        udp.register_request_listener(OpCodes::Query, RRTypes::SshFp, on_sshfp_query(&store));
        udp.register_request_listener(OpCodes::Query, RRTypes::Smimea, on_smimea_query(&store));
        udp.register_request_listener(OpCodes::Query, RRTypes::Https, on_https_query(&store));
        udp.register_request_listener(OpCodes::Query, RRTypes::Svcb, on_svcb_query(&store));
        udp.register_request_listener(OpCodes::Query, RRTypes::Uri, on_uri_query(&store));
        udp.register_request_listener(OpCodes::Query, RRTypes::Loc, on_loc_query(&store));
        
        udp.register_request_listener(OpCodes::Query, RRTypes::Soa, on_soa_query(&store));
        udp.register_request_listener(OpCodes::Query, RRTypes::Any, on_any_query(&store));*/

        let tcp = TcpServer::new();
        tcp.register_request_listener(OpCodes::Query, RRTypes::A, on_a_query(&store));
        tcp.register_request_listener(OpCodes::Query, RRTypes::Aaaa, on_a_query(&store));/*
        tcp.register_request_listener(OpCodes::Query, RRTypes::Ns, on_ns_query(&store));
        tcp.register_request_listener(OpCodes::Query, RRTypes::Txt, on_txt_query(&store));
        tcp.register_request_listener(OpCodes::Query, RRTypes::Mx, on_mx_query(&store)); //TEST
        tcp.register_request_listener(OpCodes::Query, RRTypes::Ptr, on_ptr_query(&store));
        tcp.register_request_listener(OpCodes::Query, RRTypes::CName, on_cname_query(&store));
        tcp.register_request_listener(OpCodes::Query, RRTypes::Srv, on_srv_query(&store));
        tcp.register_request_listener(OpCodes::Query, RRTypes::Naptr, on_naptr_query(&store));
        tcp.register_request_listener(OpCodes::Query, RRTypes::SshFp, on_sshfp_query(&store));
        tcp.register_request_listener(OpCodes::Query, RRTypes::Smimea, on_smimea_query(&store));
        tcp.register_request_listener(OpCodes::Query, RRTypes::Https, on_https_query(&store));
        tcp.register_request_listener(OpCodes::Query, RRTypes::Svcb, on_svcb_query(&store));
        tcp.register_request_listener(OpCodes::Query, RRTypes::Uri, on_uri_query(&store));
        tcp.register_request_listener(OpCodes::Query, RRTypes::Loc, on_loc_query(&store));

        tcp.register_request_listener(OpCodes::Query, RRTypes::Soa, on_soa_query(&store));
        tcp.register_request_listener(OpCodes::Query, RRTypes::Axfr, on_axfr_query(&store));
        tcp.register_request_listener(OpCodes::Query, RRTypes::Ixfr, on_ixfr_query(&store));
        tcp.register_request_listener(OpCodes::Query, RRTypes::Any, on_any_query(&store));*/

        Self {
            store,
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
        self.store.write().unwrap().open(file_path, domain)
    }

    //pub fn register_journal(&mut self, file_path: &str, domain: &str) -> io::Result<()> {
    //    self.zones.write().unwrap().set_journal_for(domain, Journal::open(file_path)?)
    //}
}
