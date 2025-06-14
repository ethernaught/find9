mod rpc;
mod utils;
mod dns;
mod zone;
//mod unix_rpc;

use std::{io, thread};
use crate::dns::dns::Dns;

pub const MAX_CNAME_CHAIN_SIZE: u8 = 10;
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

//elisabeth.ns.cloudflare.com

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



SOA

When a server receives a query:

It tries to find the best match for the query name in its zones.

If the exact name doesn't exist, the server returns an NXDOMAIN with the SOA record of the nearest enclosing zone.


- OUR USE OF NxDomain & Refused may be slightly off...








TODO REQUIRED
Type	Name	Purpose
SOA	Start of Authority	Identifies the zone's authoritative info and controls zone transfers.
NS	Name Server	Specifies the authoritative name servers for the zone.
A	Address (IPv4)	Maps domain names to IPv4 addresses.
AAAA	Address (IPv6)	Maps domain names to IPv6 addresses.



TODO EXPECTED
x  MX	Mail Exchange	Specifies mail servers for email delivery.
CNAME	Canonical Name	Alias one domain to another.
x  TXT	Text	Stores human-readable data (SPF, DKIM, etc).
PTR	Pointer	Used in reverse DNS (served from reverse zones).
SRV	Service Locator	Maps services (e.g., SIP, LDAP) to target hosts and ports.
DNSKEY / DS	DNSSEC Records	Needed if DNSSEC is enabled.
CAA	Certification Authority Authorization	Limits which CAs can issue certs for a domain.
IXFR
AXFR




brad@brads-pc:~/Downloads/done$ dig @elisabeth.ns.cloudflare.com x2.find9.net NS

; <<>> DiG 9.18.30-0ubuntu0.22.04.2-Ubuntu <<>> @elisabeth.ns.cloudflare.com x2.find9.net NS
; (6 servers found)
;; global options: +cmd
;; Got answer:
;; ->>HEADER<<- opcode: QUERY, status: NOERROR, id: 34998
;; flags: qr aa rd; QUERY: 1, ANSWER: 0, AUTHORITY: 1, ADDITIONAL: 1
;; WARNING: recursion requested but not available

;; OPT PSEUDOSECTION:
; EDNS: version: 0, flags:; udp: 1232
;; QUESTION SECTION:
;x2.find9.net.			IN	NS

;; AUTHORITY SECTION:
find9.net.		1800	IN	SOA	elisabeth.ns.cloudflare.com. dns.cloudflare.com. 2375191312 10000 2400 604800 1800

;; Query time: 61 msec
;; SERVER: 162.159.38.224#53(elisabeth.ns.cloudflare.com) (UDP)
;; WHEN: Sat Jun 14 13:25:31 MDT 2025
;; MSG SIZE  rcvd: 108


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
    dns.register_zone("res/nine.zone", "nine")?;
    dns.register_zone("res/find.nine.zone", "find.nine")?;
    dns.register_zone("res/find9.net.zone", "find9.net")?;
    //dns.get_server().add_fallback(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 53));
    dns.start(6767)?;

    //println!("UDP Server started on port: {}", dns.get_udp().socket.as_ref().unwrap().local_addr().unwrap().port());
    //println!("TCP Server started on port: {}", dns.get_tcp().socket.as_ref().unwrap().local_addr().unwrap().port());

    //let mut unix_rpc = UnixRpc::new()?;
    //unix_rpc.set_database(database.clone());
    //unix_rpc.start()?.join().unwrap();

    thread::park();
    Ok(())
}

