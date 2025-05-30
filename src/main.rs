mod rpc;
mod utils;
mod dns;
mod zone;
//mod unix_rpc;

use std::io;
use crate::dns::dns::Dns;

const MAX_CNAME_CHAIN_SIZE: u8 = 10;

//pub type RecordMap = HashMap<String, HashMap<RRTypes, Vec<Box<dyn RecordBase>>>>;

//dig @127.0.0.1 -p 6767 find9.net
//NS
//+bufsize=1024
//+tcp

//REDO DNS / SERVER STUFF
//REDO UNIX-RPC


//USE ZONE FILE FORMAT
//https://en.wikipedia.org/wiki/Zone_file

//STORE ZONES IN:
// - /etc/find

//MAKE SURE WE CACHE RECORDS IN MEMORY

//if client adds opt - and UDP and size greater than 512 bytes add OPT RR
/*
You should always set TC = 1 if the response exceeds the allowed UDP size, regardless of OPT presence.

so basic jist is check for OPT in query
if exists
- add records until it wont fit
- set truncated
- send
else
- add records until it wont fit - 512

let max_udp_size = if let Some(opt) = query.opt_record {
    opt.udp_payload_size
} else {
    512
};

let response_bytes = response.to_bytes();
if response_bytes.len() > max_udp_size {
    response.set_truncated(true);
    truncate_answers(...); // optional
}
*/

//SHOULD WE HANDLE MORE THAN 1 QUERY?

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

    println!("UDP Server started on port: {}", dns.get_udp().socket.as_ref().unwrap().local_addr().unwrap().port());
    println!("TCP Server started on port: {}", dns.get_tcp().socket.as_ref().unwrap().local_addr().unwrap().port());

    //let mut unix_rpc = UnixRpc::new()?;
    //unix_rpc.set_database(database.clone());
    //unix_rpc.start()?.join().unwrap();

    loop {}
    Ok(())
}
