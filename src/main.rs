mod rpc;
mod utils;
mod dns;
mod zone;
//mod unix_rpc;

use std::collections::HashMap;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::inter::record_base::RecordBase;
use rlibdns::zone::zone_parser::ZoneParser;
use crate::dns::dns::Dns;
use crate::zone::inter::zone_types::ZoneTypes;
use crate::zone::zone::Zone;

//pub type RecordMap = HashMap<String, HashMap<RRTypes, Vec<Box<dyn RecordBase>>>>;

//dig @127.0.0.1 -p 6767 net.unet

//REDO DNS / SERVER STUFF
//REDO UNIX-RPC


//USE ZONE FILE FORMAT
//https://en.wikipedia.org/wiki/Zone_file

//STORE ZONES IN:
// - /etc/find

//MAKE SURE WE CACHE RECORDS IN MEMORY

//ZONE PARSING FORGET ORIGIN WE GIVE IF ORIGIN IS STATED WITHIN FILE

fn main() -> io::Result<()> {
    /*
    Type	Role
    master	Owns the original zone data, serves it authoritatively
    slave	Gets zone data from the master via zone transfers (AXFR, IXFR) RECORDS - TCP (SOA - SERIAL IS USED TO KNOW ABOUT UPDATES TO RECORDS - MASTER NOTIFIES SLAVES)
    stub	Only stores NS records of a zone, not full data
    forward	Forwards queries to another server (like a proxy)
    hint	Used for root servers (rarely modified)
    */

    let mut dns = Dns::new();
    dns.register_zone("/home/brad/Downloads/find9.net.test.zone", "find9.net").unwrap();
    //dns.get_server().add_fallback(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 53));
    dns.start(6767);

    //let mut unix_rpc = UnixRpc::new()?;
    //unix_rpc.set_database(database.clone());
    //unix_rpc.start()?.join().unwrap();

    loop {}
    Ok(())
}
