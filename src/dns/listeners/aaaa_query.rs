use std::collections::HashMap;
use std::io;
use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::cname_record::CNameRecord;
use crate::rpc::events::query_event::QueryEvent;
use crate::zone::zone::Zone;

pub fn on_aaaa_query(zones: &Arc<RwLock<HashMap<String, Zone>>>) -> impl Fn(&mut QueryEvent) -> io::Result<()> {
    let zones = zones.clone();

    move |event| {
        match zones.read().unwrap().get(&event.get_query().get_name()).unwrap().get_records("", &RRTypes::CName) {
            Some(records) => {
                for record in records.iter().take(3) {
                    if let Some(record) = record.as_any().downcast_ref::<CNameRecord>() {
                        let target = record.get_target().unwrap();

                        match zones.read().unwrap().get(&target).unwrap().get_records("", &RRTypes::Aaaa) {
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
                match zones.read().unwrap().get(&event.get_query().get_name()).unwrap().get_records("", &RRTypes::Aaaa) {
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
}
