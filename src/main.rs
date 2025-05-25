mod rpc;
mod utils;
mod dns;
//mod unix_rpc;

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use rlibdns::zone::zone_parser::ZoneParser;
use crate::dns::dns::Dns;

//dig @127.0.0.1 -p 6767 net.unet

//REDO DNS / SERVER STUFF
//REDO UNIX-RPC


//USE ZONE FILE FORMAT
//https://en.wikipedia.org/wiki/Zone_file

//STORE ZONES IN:
// - /etc/find

//MAKE SURE WE CACHE RECORDS IN MEMORY

fn main() -> io::Result<()> {

    let find9 = ZoneParser::new("/home/brad/find9/find9.net.zone", "find9.net");



    //let mut dns = Dns::new();
    //dns.get_server().add_fallback(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 53));
    //dns.start(6767)?;

    //let mut unix_rpc = UnixRpc::new()?;
    //unix_rpc.set_database(database.clone());
    //unix_rpc.start()?.join().unwrap();

    loop {}
    Ok(())
}
