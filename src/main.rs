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

pub type RecordMap = HashMap<String, HashMap<RRTypes, Vec<Box<dyn RecordBase>>>>;

//dig @127.0.0.1 -p 6767 net.unet

//REDO DNS / SERVER STUFF
//REDO UNIX-RPC


//USE ZONE FILE FORMAT
//https://en.wikipedia.org/wiki/Zone_file

//STORE ZONES IN:
// - /etc/find

//MAKE SURE WE CACHE RECORDS IN MEMORY

fn main() -> io::Result<()> {
    /*
    Type	Role
    master	Owns the original zone data, serves it authoritatively
    slave	Gets zone data from the master via zone transfers
    stub	Only stores NS records of a zone, not full data
    forward	Forwards queries to another server (like a proxy)
    hint	Used for root servers (rarely modified)
    */

    /*
    let mut records = RecordMap::new();

    let mut parser = ZoneParser::new("/home/brad/find9/find9.net.zone", "find9.net").unwrap();

    for (name, record) in parser.iter() {
        println!("{}: {:?}", name, record);

        records
            .entry(name)
            .or_insert_with(HashMap::new)
            .entry(record.get_type())
            .or_insert_with(Vec::new)
            .push(record);
    }
    */
    let mut zones = HashMap::new();

    let domain = "find9.net";
    let mut zone = Zone::new(ZoneTypes::Master);

    let mut parser = ZoneParser::new("/home/brad/find9/find9.net.zone", domain).unwrap();
    for (name, record) in parser.iter() {
        if !name.eq(domain){
            continue;
        }

        zone.add_record(record);
    }

    zones.insert(domain, zone);

    println!("{:?}", zones);



    let mut dns = Dns::new();
    //dns.get_server().add_fallback(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 53));
    dns.start(6767)?;

    //let mut unix_rpc = UnixRpc::new()?;
    //unix_rpc.set_database(database.clone());
    //unix_rpc.start()?.join().unwrap();

    loop {}
    Ok(())
}
