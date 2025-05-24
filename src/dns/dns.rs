use std::io;
use std::thread::JoinHandle;
use rlibdns::messages::inter::rr_types::RRTypes;
use crate::dns::listeners::a_query::on_a_query;
use crate::dns::listeners::aaaa_query::on_aaaa_query;
use crate::dns::listeners::ns_query::on_ns_query;
use crate::dns::listeners::soa_query::on_soa_query;
use crate::dns::listeners::txt_query::on_txt_query;
use crate::dns::server::Server;

pub struct Dns {
    server: Server
}

impl Dns {

    pub fn new() -> Self {
        let server = Server::new();

        //BASED ON CONFIG ENABLE SPECIFIC QUERY LISTENERS
        server.register_query_listener(RRTypes::A, on_a_query());
        server.register_query_listener(RRTypes::Aaaa, on_aaaa_query());
        server.register_query_listener(RRTypes::Ns, on_ns_query());
        server.register_query_listener(RRTypes::Txt, on_txt_query());
        server.register_query_listener(RRTypes::Soa, on_soa_query());

        Self {
            server
        }
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
}
