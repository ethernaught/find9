mod rpc;
mod utils;
mod dns;

use std::{io, thread};
use crate::dns::dns::Dns;

pub const BOGON_ALLOWED: bool = true;
pub const ANY_QUERY_ALLOWED: bool = false;
pub const MAX_CNAME_CHAIN_SIZE: u8 = 10;
pub const MAX_QUERIES: usize = 1;
pub const MAX_ANSWERS: usize = 3;
pub const COOKIE_SECRET: &[u8] = b"HELLO WORLD";


//dig @127.0.0.1 -p 6767 find9.net
//NS
//+bufsize=1024
//+tcp

//dig @192.168.0.8 find9.net -t ixfr=2025070401


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




TODO (SAME AS A, AAAA)
CAA	Certificate Authority Authorization — defined per FQDN
TLSA	Used in DANE for certificate binding
SPF	(Deprecated, use TXT) — still answered if defined
CERT	Stores certificates or related PKI data
OPENPGPKEY	For OpenPGP key discovery
SVCB/HTTPS	New service discovery records (used by modern browsers)

TODO (NOT LIKE A, AAAA)
DNSKEY/DS	Only at apex or delegation boundary
RRSIG/NSEC/NSEC3	DNSSEC-related, tied to zone structure
AXFR/IXFR	Not answered like A/AAAA — used for transfers

TODO EXPECTED
DNSKEY / DS	DNSSEC Records	Needed if DNSSEC is enabled.
CAA	Certification Authority Authorization	Limits which CAs can issue certs for a domain.


FOR AXFR AND IFXR ONLY ALLOW SPECIFIC IPS

*/

//ADD TEST CASES...

/*
WITH AXFR

SOMEHOW RECORD CHANGES FOR IXFR...

VERIFY SSHFP - THROUGH MAC AS IT WONT WORK ON LINUX...


POSSIBLY SKIP XFRS FOR NOW


AXFR SHOULD KEEP ORDER...









Maybe we shouldnt store the SOA as a record within the zone but as a part of each zone...
- as their is only 1 SOA associated with each zone - unless its a journal system for IXFR

- in the other sense just use jnl for the other SOA types...




IXFR PREPS
dig @localhost -p 6767 find9.net ixfr=2





FOR IXFR
x - we need to modify domain registering - IE opening / reading to be within zone
- we need to add encoding for JNL and zone files
- we need to add transaction consolodation
x - we need to be able to open JNL files for zones and effect SOA serial accordingly...
x - we need to add request records to the event somehow so that we can be aware of the SOA serial

*/


fn main() -> io::Result<()> {
    //ensure we fix bugs with odd queries IE TLD parsing

    //OPT bug fixing
    //cookie - server-gen - HMAC(client IP + client cookie + server secret).

    //hmac_sha256("test".as_bytes(), "test".as_bytes());


    let mut dns = Dns::new();
    dns.register_zone("res/nine.zone", "nine")?;
    dns.register_zone("res/find.nine.zone", "find.nine")?;
    dns.register_zone("res/find9.net.zone", "find9.net")?;
    dns.register_zone("res/sub.find9.net.zone", "sub.find9.net")?;
    dns.register_zone("res/192.168.0.zone", "0.168.192.in-addr.arpa")?;
    dns.register_journal("res/find9.net.zone.jnl", "find9.net")?;
    //dns.get_server().add_fallback(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 53));
    dns.start(6767)?;

    println!();
    println!();
    println!();
    println!();
    //dns.test();

    //println!("UDP Server started on port: {}", dns.get_udp().socket.as_ref().unwrap().local_addr().unwrap().port());
    //println!("TCP Server started on port: {}", dns.get_tcp().socket.as_ref().unwrap().local_addr().unwrap().port());

    //let mut unix_rpc = UnixRpc::new()?;
    //unix_rpc.set_database(database.clone());
    //unix_rpc.start()?.join().unwrap();

    thread::park();
    Ok(())
}
