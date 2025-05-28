use std::collections::HashMap;
use std::io;
use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::cname_record::CNameRecord;
use crate::rpc::events::query_event::QueryEvent;
use crate::zone::zone::Zone;

pub fn on_ns_query(zones: &Arc<RwLock<HashMap<String, Zone>>>) -> impl Fn(&mut QueryEvent) -> io::Result<()> {
    let zones = zones.clone();

    move |event| {
        match zones.read().unwrap().get(&event.get_query().get_name()).unwrap().get_records("", &RRTypes::CName) {
            Some(records) => {
                for record in records.iter().take(3) {
                    if let Some(record) = record.as_any().downcast_ref::<CNameRecord>() {
                        let target = record.get_target().unwrap();

                        match zones.read().unwrap().get(&target).unwrap().get_records("", &RRTypes::Ns) {
                            Some(records) => {
                                for record in records.iter().take(3) {
                                    event.add_answer(&target, record.clone());
                                }
                            }
                            None => return Err(io::Error::new(io::ErrorKind::Other, "Document not found"))
                        }
                    }
                }
            }
            None => {
                match zones.read().unwrap().get(&event.get_query().get_name()).unwrap().get_records("", &RRTypes::Ns) {
                    Some(records) => {
                        for record in records.iter().take(3) {
                            event.add_answer(&event.get_query().get_name(), record.clone());
                        }
                    }
                    None => return Err(io::Error::new(io::ErrorKind::Other, "Document not found"))
                }
            }
        }

        Ok(())
    }
    //move |event| {
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
                    "ns",
                    Some(vec!["class", "ttl", "server", "network"]),
                    Some(format!("class = {} AND name = '{}' AND {}", class.get_code(), target, is_bogon).as_str()),
                );

                if records.is_empty() {
                    return Err(io::Error::new(io::ErrorKind::Other, "Document not found"));
                }

                for record in records {
                    let ttl = record.get("ttl").unwrap().parse::<u32>().unwrap();
                    let server = record.get("server").unwrap().as_str();

                    event.add_answer(&target, Box::new(NsRecord::new(class, ttl, server)));
                }
            }

        } else {
            let records = database.get(
                "ns",
                Some(vec!["class", "ttl", "server", "network"]),
                Some(format!("class = {} AND name = '{}' AND {}", class.get_code(), name, is_bogon).as_str()),
            );

            if records.is_empty() {
                return Err(io::Error::new(io::ErrorKind::Other, "Document not found"));
            }

            for record in records {
                let ttl = record.get("ttl").unwrap().parse::<u32>().unwrap();
                let server = record.get("server").unwrap().as_str();

                event.add_answer(&name, Box::new(NsRecord::new(class, ttl, server)));
            }
        }
        */
        
    //    Ok(())
    //}
}
