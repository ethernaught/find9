use std::io;
use std::net::IpAddr;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::inter::opt_codes::OptCodes;
use rlibdns::records::opt_record::OptRecord;
use crate::COOKIE_SECRET;
use crate::rpc::events::query_event::QueryEvent;
use crate::utils::hash::hmac::hmac;
use crate::utils::hash::sha256::Sha256;

pub fn opt_record() -> impl Fn(&mut QueryEvent) -> io::Result<()> {

    move |event| {

        /*
        let mut record = records.get_mut(0).unwrap();
        if !record.get_type().eq(&RRTypes::Opt) {
            continue;
        }

        let mut record = record.as_any_mut().downcast_mut::<OptRecord>().unwrap();

        if let Some(cookie) = record.get_option(&OptCodes::Cookie) {
            match cookie.len() {
                8 => { //CLIENT ONLY
                    let hmac = match src_addr.ip() {
                        IpAddr::V4(addr) => hmac::<Sha256>(COOKIE_SECRET, &[cookie, addr.octets().as_slice()].concat()),
                        IpAddr::V6(addr) => hmac::<Sha256>(COOKIE_SECRET, &[cookie, addr.octets().as_slice()].concat())
                    };

                    let mut c = vec![0u8; 24];
                    c.extend_from_slice(cookie);
                    c.extend_from_slice(&hmac[..16]);

                }
                24 => { //CLIENT + SERVER
                    let client_cookie = &cookie[..8];
                    let server_cookie = &cookie[8..];

                    //VERIFY SERVER_COOKIE - EXISTS
                    if server_cookie.eq("".as_bytes()) {
                        // Valid — echo the same cookie

                    } else {
                        let hmac = match src_addr.ip() {
                            IpAddr::V4(addr) => hmac::<Sha256>(COOKIE_SECRET, &[client_cookie, addr.octets().as_slice()].concat()),
                            IpAddr::V6(addr) => hmac::<Sha256>(COOKIE_SECRET, &[client_cookie, addr.octets().as_slice()].concat())
                        };

                        let mut c = vec![0u8; 24];
                        c.extend_from_slice(cookie);
                        c.extend_from_slice(&hmac[..16]);
                    }
                }
                _ => unimplemented!()
            }
            println!("{:x?}", cookie);
        }
        */


        Ok(())
    }
}
