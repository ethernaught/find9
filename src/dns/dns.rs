use std::io;
use std::net::SocketAddr;
use std::thread::JoinHandle;
use rlibdns::messages::inter::record_types::RecordTypes;
use crate::database::sqlite::Database;
use crate::dns::listeners::a_query::on_a_query;
use crate::dns::listeners::aaaa_query::on_aaaa_query;
use crate::dns::listeners::ns_query::on_ns_query;
use crate::dns::listeners::txt_query::on_txt_query;
use crate::dns::server::Server;

pub struct Dns {
    server: Server
}

impl Dns {

    pub fn new(database: &Database) -> Self {
        let mut server = Server::new();

        server.register_request_listener(RecordTypes::A, on_a_query(database));
        server.register_request_listener(RecordTypes::Aaaa, on_aaaa_query(database));
        server.register_request_listener(RecordTypes::Ns, on_ns_query(database));
        server.register_request_listener(RecordTypes::Txt, on_txt_query(database));

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
