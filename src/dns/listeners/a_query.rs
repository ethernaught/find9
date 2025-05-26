use std::io;
use std::net::Ipv4Addr;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::a_record::ARecord;
use rlibdns::records::cname_record::CNameRecord;
use crate::RecordMap;
use crate::rpc::events::query_event::QueryEvent;

pub fn on_a_query() -> impl Fn(&mut QueryEvent) -> io::Result<()> {
    let zones = RecordMap::new();
    move |event| {
        let records = zones.get(&event.get_query().get_name()).unwrap().get(&RRTypes::CName).unwrap();

        if records.is_empty() {
            let records = zones.get(&event.get_query().get_name()).unwrap().get(&RRTypes::A).unwrap();

            if records.is_empty() {
                //NO A...
            }

            for record in records {
                //let ttl = record.get("ttl").unwrap().parse::<u32>().unwrap();
                //let address = record.get("address").unwrap().parse::<u32>().unwrap();

                event.add_answer(&event.get_query().get_name(), record);
            }


        } else {
            for record in records {
                if let Some(record) = record.as_any().downcast_ref::<CNameRecord>() {
                    let records = zones.get(&event.get_query().get_name()).unwrap().get(&RRTypes::A).unwrap();

                    if records.is_empty() {
                        //NO A...
                    }

                    let target = record.get_target().unwrap();

                    for record in records {
                        event.add_answer(&target, record);
                    }
                }
            }
        }


        /*
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
                    "a",
                    Some(vec!["class", "ttl", "address", "network"]),
                    Some(format!("class = {} AND name = '{}' AND {}", class.get_code(), target, is_bogon).as_str()),
                );

                if records.is_empty() {
                    return Err(io::Error::new(io::ErrorKind::Other, "Document not found"));
                }

                for record in records {
                    let ttl = record.get("ttl").unwrap().parse::<u32>().unwrap();
                    let address = record.get("address").unwrap().parse::<u32>().unwrap();

                    event.add_answer(&target, Box::new(ARecord::new(class, false, ttl, Ipv4Addr::from(address))));
                }
            }

        } else {
            let records = database.get(
                "a",
                Some(vec!["class", "ttl", "address", "network"]),
                Some(format!("class = {} AND name = '{}' AND {}", class.get_code(), name, is_bogon).as_str()),
            );

            if records.is_empty() {
                return Err(io::Error::new(io::ErrorKind::Other, "Document not found"));
            }

            for record in records {
                let ttl = record.get("ttl").unwrap().parse::<u32>().unwrap();
                let address = record.get("address").unwrap().parse::<u32>().unwrap();

                event.add_answer(&name, Box::new(ARecord::new(class, false, ttl, Ipv4Addr::from(address))));
            }
        }
        */

        Ok(())
    }
}
