use std::io;
use std::net::Ipv6Addr;
use rlibdns::records::aaaa_record::AaaaRecord;
use rlibdns::records::cname_record::CNameRecord;
use crate::database::sqlite::Database;
use crate::rpc::events::query_event::QueryEvent;

pub fn on_aaaa_query(database: &Database) -> impl Fn(&mut QueryEvent) -> io::Result<()> {
    let database = database.clone();
    move |event| {
        //let is_bogon = is_bogon(message.get_origin().unwrap());//if  { "network < 2" } else { "network > 0" };
        let is_bogon = "network < 2";
        let name = event.get_query().get_name().to_lowercase();
        let class = event.get_query().get_dns_class();

        let records = database.get(
            "cname",
            Some(vec!["class", "ttl", "target", "network"]),
            Some(format!("class = {} AND name = '{}' AND {}", class.get_code(), name, is_bogon).as_str()),
        );

        if !records.is_empty() {
            for record in records {
                let ttl = record.get("ttl").unwrap().parse::<u32>().unwrap();
                let target = record.get("target").unwrap().as_str();
                event.add_answer(&name, Box::new(CNameRecord::new(class, ttl, target)));

                let records = database.get(
                    "aaaa",
                    Some(vec!["class", "ttl", "address", "network"]),
                    Some(format!("class = {} AND name = '{}' AND {}", class.get_code(), target, is_bogon).as_str()),
                );

                if records.is_empty() {
                    return Err(io::Error::new(io::ErrorKind::Other, "Document not found"));
                }

                for record in records {
                    let ttl = record.get("ttl").unwrap().parse::<u32>().unwrap();
                    let address = record.get("address").unwrap().parse::<u128>().unwrap();

                    event.add_answer(&target, Box::new(AaaaRecord::new(class, false, ttl, Ipv6Addr::from(address))));
                }
            }

        } else {
            let records = database.get(
                "aaaa",
                Some(vec!["class", "ttl", "address", "network"]),
                Some(format!("class = {} AND name = '{}' AND {}", class.get_code(), name, is_bogon).as_str()),
            );

            if records.is_empty() {
                return Err(io::Error::new(io::ErrorKind::Other, "Document not found"));
            }

            for record in records {
                let ttl = record.get("ttl").unwrap().parse::<u32>().unwrap();
                let address = record.get("address").unwrap().parse::<u128>().unwrap();

                event.add_answer(&name, Box::new(AaaaRecord::new(class, false, ttl, Ipv6Addr::from(address))));
            }
        }

        Ok(())
    }
}
