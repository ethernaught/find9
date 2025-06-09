mod rpc;
mod utils;
mod dns;
mod zone;
//mod unix_rpc;

use std::io;
use crate::dns::dns::Dns;

pub const MAX_CNAME_CHAIN_SIZE: u8 = 2;
pub const MAX_QUERIES: usize = 1;
pub const MAX_ANSWERS: usize = 3;
pub const COOKIE_SECRET: &[u8] = b"HELLO WORLD";

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





CNAME CHAIN WITH NO END
brad@brad-lp:~$ dig @elisabeth.ns.cloudflare.com x1.find9.net

; <<>> DiG 9.18.30-0ubuntu0.22.04.2-Ubuntu <<>> @elisabeth.ns.cloudflare.com x1.find9.net
; (6 servers found)
;; global options: +cmd
;; Got answer:
;; ->>HEADER<<- opcode: QUERY, status: NXDOMAIN, id: 31131
;; flags: qr aa rd; QUERY: 1, ANSWER: 0, AUTHORITY: 1, ADDITIONAL: 1
;; WARNING: recursion requested but not available

;; OPT PSEUDOSECTION:
; EDNS: version: 0, flags:; udp: 1232
;; QUESTION SECTION:
;x1.find9.net.			IN	A

;; AUTHORITY SECTION:
find9.net.		1800	IN	SOA	elisabeth.ns.cloudflare.com. dns.cloudflare.com. 2374971435 10000 2400 604800 1800

;; Query time: 34 msec
;; SERVER: 108.162.194.224#53(elisabeth.ns.cloudflare.com) (UDP)
;; WHEN: Mon Jun 09 10:15:54 MDT 2025
;; MSG SIZE  rcvd: 108





PROPER CNAME RESPONSE
brad@brad-lp:~$ dig @elisabeth.ns.cloudflare.com x1.find9.net

; <<>> DiG 9.18.30-0ubuntu0.22.04.2-Ubuntu <<>> @elisabeth.ns.cloudflare.com x1.find9.net
; (6 servers found)
;; global options: +cmd
;; Got answer:
;; ->>HEADER<<- opcode: QUERY, status: NOERROR, id: 27890
;; flags: qr aa rd; QUERY: 1, ANSWER: 9, AUTHORITY: 0, ADDITIONAL: 1
;; WARNING: recursion requested but not available

;; OPT PSEUDOSECTION:
; EDNS: version: 0, flags:; udp: 1232
;; QUESTION SECTION:
;x1.find9.net.			IN	A

;; ANSWER SECTION:
x1.find9.net.		300	IN	CNAME	x2.find9.net.
x2.find9.net.		300	IN	CNAME	x3.find9.net.
x3.find9.net.		300	IN	CNAME	x4.find9.net.
x4.find9.net.		300	IN	CNAME	x5.find9.net.
x5.find9.net.		300	IN	CNAME	x6.find9.net.
x6.find9.net.		300	IN	CNAME	x7.find9.net.
x7.find9.net.		300	IN	CNAME	x8.find9.net.
x8.find9.net.		300	IN	CNAME	x9.find9.net.
x9.find9.net.		300	IN	A	127.0.0.1

;; Query time: 65 msec
;; SERVER: 162.159.38.224#53(elisabeth.ns.cloudflare.com) (UDP)
;; WHEN: Mon Jun 09 10:18:09 MDT 2025
;; MSG SIZE  rcvd: 193


EXESSIVE CNAME CHAIN
brad@brad-lp:~$ dig @elisabeth.ns.cloudflare.com x1.find9.net

; <<>> DiG 9.18.30-0ubuntu0.22.04.2-Ubuntu <<>> @elisabeth.ns.cloudflare.com x1.find9.net
; (6 servers found)
;; global options: +cmd
;; Got answer:
;; ->>HEADER<<- opcode: QUERY, status: SERVFAIL, id: 10804
;; flags: qr rd; QUERY: 1, ANSWER: 0, AUTHORITY: 0, ADDITIONAL: 1
;; WARNING: recursion requested but not available

;; OPT PSEUDOSECTION:
; EDNS: version: 0, flags:; udp: 1232
;; QUESTION SECTION:
;x1.find9.net.			IN	A

;; Query time: 55 msec
;; SERVER: 172.64.34.224#53(elisabeth.ns.cloudflare.com) (UDP)
;; WHEN: Mon Jun 09 10:20:28 MDT 2025
;; MSG SIZE  rcvd: 41

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

    //MAY BE ISSUE WITH AA - AUTHORITY ALWAYS BEING TRUE...?

    //ensure we fix bugs with odd queries IE TLD parsing

    //OPT bug fixing
    //cookie - server-gen - HMAC(client IP + client cookie + server secret).

    //hmac_sha256("test".as_bytes(), "test".as_bytes());


    let mut dns = Dns::new();
    dns.register_zone("res/find9.net.zone", "find9.net")?;
    //dns.get_server().add_fallback(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 53));
    dns.start(6767)?;

    //println!("UDP Server started on port: {}", dns.get_udp().socket.as_ref().unwrap().local_addr().unwrap().port());
    //println!("TCP Server started on port: {}", dns.get_tcp().socket.as_ref().unwrap().local_addr().unwrap().port());

    //let mut unix_rpc = UnixRpc::new()?;
    //unix_rpc.set_database(database.clone());
    //unix_rpc.start()?.join().unwrap();

    loop {}
    Ok(())
}

