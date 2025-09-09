use std::io;
use std::net::IpAddr;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::inter::opt_codes::OptCodes;
use rlibdns::records::opt_record::OptRecord;
use crate::COOKIE_SECRET;
use crate::rpc::events::query_event::RequestEvent;
use crate::utils::hash::hmac::hmac;
use crate::utils::hash::sha256::Sha256;

pub fn opt_record() -> impl Fn(&mut RequestEvent) -> io::Result<()> {

    move |event| {

        /*
        let mut record = records.get_mut(0).unwrap();
        if !record.get_type().eq(&RRTypes::Opt) {
            continue;
        }

        let record = record.as_any().downcast_ref::<OptRecord>().unwrap();

        if let Some(cookie) = record.get_option(&OptCodes::Cookie) {
            match cookie.len() {
                8 => { //CLIENT ONLY
                    let hmac = match src_addr.ip() {
                        IpAddr::V4(addr) => hmac::<Sha256>(COOKIE_SECRET, &[cookie, addr.octets().as_slice()].concat()),
                        IpAddr::V6(addr) => hmac::<Sha256>(COOKIE_SECRET, &[cookie, addr.octets().as_slice()].concat())
                    };

                    let mut c = vec![0u8; 24];
                    c[..8].copy_from_slice(cookie);
                    c[8..].copy_from_slice(&hmac[..16]);
                    println!("UNVERIFIED");

                    let mut record = OptRecord::new(record.get_payload_size(), record.get_ext_rcode(), record.get_edns_version(), record.get_flags());
                    record.insert_option(OptCodes::Cookie, c);
                    response.add_additional_record("", record.upcast());

                }
                24 => { //CLIENT + SERVER
                    let client_cookie = &cookie[..8];
                    let server_cookie = &cookie[8..];

                    let hmac = match src_addr.ip() {
                        IpAddr::V4(addr) => hmac::<Sha256>(COOKIE_SECRET, &[client_cookie, addr.octets().as_slice()].concat()),
                        IpAddr::V6(addr) => hmac::<Sha256>(COOKIE_SECRET, &[client_cookie, addr.octets().as_slice()].concat())
                    };

                    //VERIFY SERVER_COOKIE - EXISTS
                    if server_cookie == &hmac[..16] {
                        // Valid — echo the same cookie
                        println!("VERIFIED");

                        let mut record = OptRecord::new(record.get_payload_size(), record.get_ext_rcode(), record.get_edns_version(), record.get_flags());
                        record.insert_option(OptCodes::Cookie, cookie.to_vec());
                        response.add_additional_record("", record.upcast());

                    } else {
                        let mut c = vec![0u8; 24];
                        c[..8].copy_from_slice(cookie);
                        c[8..].copy_from_slice(&hmac[..16]);

                        let mut record = OptRecord::new(record.get_payload_size(), record.get_ext_rcode(), record.get_edns_version(), record.get_flags());
                        record.insert_option(OptCodes::Cookie, c);
                        response.add_additional_record("", record.upcast());

                        println!("FALSE COOKIE");
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
