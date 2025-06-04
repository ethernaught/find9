use std::collections::HashMap;
use std::io;
use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::cname_record::CNameRecord;
use crate::rpc::events::query_event::QueryEvent;
use crate::utils::query_utils::chain_cname;
use crate::zone::zone::Zone;

pub fn on_soa_query(zones: &Arc<RwLock<HashMap<String, Zone>>>) -> impl Fn(&mut QueryEvent) -> io::Result<()> {
    let zones = zones.clone();

    move |event| {
        /*
        let (name, tld) = match split_domain(&event.get_query().get_name()) {
            Some((name, tld)) => (name, tld),
            None => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid query name"))
        };

        match zones.read().unwrap().get(&tld).unwrap().get_records(&name, &RRTypes::CName) {
            Some(records) => {
                let record = records.get(0).unwrap();
                event.add_answer(&event.get_query().get_name(), record.clone());
                chain_cname(&zones, event, record.as_any().downcast_ref::<CNameRecord>().unwrap(), 0);
            }
            None => {
                match zones.read().unwrap().get(&tld).unwrap().get_records(&name, &event.get_query().get_type()) {
                    Some(records) => {
                        for record in records.iter().take(3) {
                            event.add_answer(&tld, record.clone());
                        }
                    }
                    None => return Err(io::Error::new(io::ErrorKind::Other, "Document not found"))
                }
            }
        }
        */

        Ok(())



        /*
        //let is_bogon = is_bogon(message.get_origin().unwrap());//if  { "network < 2" } else { "network > 0" };
        let is_bogon = "network < 2";
        let name = event.get_query().get_name().to_lowercase();
        let class = event.get_query().get_dns_class();

        let records = database.get(
            "soa",
            Some(vec!["class", "ttl", "domain", "mailbox", "serial_number", "refresh_interval", "retry_interval", "expire_limit", "minimum_ttl", "network"]),
            Some(format!("class = {} AND name = '{}' AND {}", class.get_code(), name, is_bogon).as_str()),
        );

        if records.is_empty() {
            return Err(io::Error::new(io::ErrorKind::Other, "Document not found"));
        }

        for record in records {
            let ttl = record.get("ttl").unwrap().parse::<u32>().unwrap();
            let domain = record.get("domain").unwrap().as_str();
            let mailbox = record.get("mailbox").unwrap().as_str();
            let serial_number = record.get("serial_number").unwrap().parse::<u32>().unwrap();
            let refresh_interval = record.get("refresh_interval").unwrap().parse::<u32>().unwrap();
            let retry_interval = record.get("retry_interval").unwrap().parse::<u32>().unwrap();
            let expire_limit = record.get("expire_limit").unwrap().parse::<u32>().unwrap();
            let minimum_ttl = record.get("minimum_ttl").unwrap().parse::<u32>().unwrap();

            event.add_answer(&name, Box::new(SoaRecord::new(class, ttl, domain, mailbox, serial_number, refresh_interval, retry_interval, expire_limit, minimum_ttl)));
        }
        */
    }
}
