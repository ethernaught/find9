mod database;
mod rpc;
mod utils;
mod dns;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::os::unix::net::UnixListener;
use crate::dns::Dns;

//dig @127.0.0.1 -p 6767 net.unet

fn main() {
    let mut dns = Dns::new();
    dns.add_fallback(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 53));
    dns.set_database("records.db");
    dns.start(6767).unwrap();


    let listener = UnixListener::bind("/tmp/find9.sock").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {

            },
            Err(_) => {}
        }
    }





    loop {}
}
