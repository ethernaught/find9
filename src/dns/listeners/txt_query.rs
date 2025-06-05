use std::io;
use std::sync::{Arc, RwLock};
use crate::MAX_ANSWERS;
use crate::rpc::events::query_event::QueryEvent;
use crate::utils::query_utils::chain_cname;
use crate::zone::zone::Zone;

pub fn on_txt_query(zones: &Arc<RwLock<Zone>>) -> impl Fn(&mut QueryEvent) -> io::Result<()> {
    let zones = zones.clone();

    move |event| {
        let name = match chain_cname(&zones, &event.get_query().get_name(), 0) {
            Some(name) => name,
            None => event.get_query().get_name()
        };

        match zones.read().unwrap().get_deepest_zone(&name) {
            Some(zone) => {
                event.set_authoritative(true);

                match zone.get_records(&event.get_query().get_type()) {
                    Some(records) => {
                        for record in records.iter().take(MAX_ANSWERS) {
                            event.add_answer(&event.get_query().get_name(), record.clone());
                        }
                    }
                    None => return Err(io::Error::new(io::ErrorKind::Other, "Document not found"))
                }
            }
            None => event.set_authoritative(false)
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
            "txt",
            Some(vec!["class", "ttl", "content", "network"]),
            Some(format!("class = {} AND name = '{}' AND {}", class.get_code(), name, is_bogon).as_str()),
        );

        if records.is_empty() {
            return Err(io::Error::new(io::ErrorKind::Other, "Document not found"));
        }

        for record in records {
            let ttl = record.get("ttl").unwrap().parse::<u32>().unwrap();
            let content = record.get("content").unwrap().as_str();

            event.add_answer(&name, Box::new(TxtRecord::new(class, false, ttl, vec![content.to_string()])));
        }
        */

    //    Ok(())
    //}
}
