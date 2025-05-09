mod database;
mod rpc;
mod utils;
mod dns_ext;
mod dns;
mod unix_rpc;

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use rlibdns::messages::inter::record_types::RecordTypes;
use crate::database::sqlite::Database;
use crate::dns::Dns;
use crate::unix_rpc::UnixRpc;
//dig @127.0.0.1 -p 6767 net.unet

//SWITCH TO BENCODE 2

fn main() -> io::Result<()> {
    let database = Database::open_or_create("records.db")?;

    let mut dns = Dns::new();
    dns.add_fallback(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 53));
    dns.set_database(database.clone());

    dns.register_request_listener(RecordTypes::A, |request| {



    });

    dns.start(6767)?;

    let mut unix_rpc = UnixRpc::new()?;
    unix_rpc.set_database(database.clone());
    unix_rpc.start()?.join().unwrap();
    Ok(())
}
