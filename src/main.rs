mod database;
mod rpc;
mod utils;
mod dns_ext;
mod dns;
mod unix_rpc;

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use rlibdns::messages::inter::record_types::RecordTypes;
use rlibdns::records::a_record::ARecord;
use rlibdns::records::cname_record::CNameRecord;
use crate::database::sqlite::Database;
use crate::dns::Dns;
use crate::unix_rpc::UnixRpc;
//dig @127.0.0.1 -p 6767 net.unet

//SWITCH TO BENCODE 2

fn main() -> io::Result<()> {
    let database = Database::open_or_create("records.db")?;

    let mut dns = Dns::new();
    dns.add_fallback(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 53));
    //dns.set_database(database.clone());

    dns.register_request_listener(RecordTypes::A, |event| {
        
        
        /*
        let records = database.get(
            "cname",
            Some(vec!["class", "ttl", "target", "network"]),
            Some(format!("class = {} AND name = '{}' AND {}", query.get_dns_class().get_code(), query.get_query().unwrap().to_lowercase(), is_bogon).as_str())
        );
        response.add_query(query.clone());

        if !records.is_empty() {
            for record in records {
                let ttl = record.get("ttl").unwrap().parse::<u32>().unwrap();
                let target = record.get("target").unwrap().to_string();
                response.add_answers(query.get_query().unwrap(), Box::new(CNameRecord::new(query.get_dns_class(), ttl, &target)));

                let records = database.get(
                    "a",
                    Some(vec!["class", "ttl", "address", "network"]),
                    Some(format!("class = {} AND name = '{}' AND {}", query.get_dns_class().get_code(), target, is_bogon).as_str())
                );

                if records.is_empty() {
                    return Err(io::Error::new(io::ErrorKind::Other, "Document not found"));
                }

                for record in records {
                    let ttl = record.get("ttl").unwrap().parse::<u32>().unwrap();
                    let address = record.get("address").unwrap().parse::<u32>().unwrap();

                    response.add_answers(&target, Box::new(ARecord::new(query.get_dns_class(), false, ttl, Ipv4Addr::from(address))));
                }
            }

        } else {
            let records = database.get(
                "a",
                Some(vec!["class", "ttl", "address", "network"]),
                Some(format!("class = {} AND name = '{}' AND {}", query.get_dns_class().get_code(), query.get_query().unwrap().to_lowercase(), is_bogon).as_str())
            );

            if records.is_empty() {
                return Err(io::Error::new(io::ErrorKind::Other, "Document not found"));
            }

            for record in records {
                let ttl = record.get("ttl").unwrap().parse::<u32>().unwrap();
                let address = record.get("address").unwrap().parse::<u32>().unwrap();

                response.add_answers(query.get_query().unwrap(), Box::new(ARecord::new(query.get_dns_class(), false, ttl, Ipv4Addr::from(address))));
            }
        }
        */

        Ok(())
    });

    dns.start(6767)?;

    let mut unix_rpc = UnixRpc::new()?;
    unix_rpc.set_database(database.clone());
    unix_rpc.start()?.join().unwrap();
    Ok(())
}
