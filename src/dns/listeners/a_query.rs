use std::collections::HashMap;
use std::io;
use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::cname_record::CNameRecord;
use rlibdns::records::inter::record_base::RecordBase;
use crate::rpc::events::query_event::QueryEvent;
use crate::utils::domain_utils::split_domain;
use crate::zone::zone::Zone;

pub fn on_a_query(zones: &Arc<RwLock<HashMap<String, Zone>>>) -> impl Fn(&mut QueryEvent) -> io::Result<()> {
    let zones = zones.clone();

    move |event| {
        let (name, tld) = split_domain(&event.get_query().get_name()).unwrap();

        match zones.read().unwrap().get(&tld).unwrap().get_records(&name, &RRTypes::CName) {
            Some(records) => {

                //WE NEED TO ADD CHAINING CNAMES

                for record in records.iter().take(3) {
                    if let Some(record) = record.as_any().downcast_ref::<CNameRecord>() {
                        let target = record.get_target().unwrap();
                        let (name, tld) = split_domain(&target).unwrap();
                        event.add_answer(&event.get_query().get_name(), record.clone().upcast());

                        match zones.read().unwrap().get(&tld).unwrap().get_records(&name, &RRTypes::A) {
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
                match zones.read().unwrap().get(&tld).unwrap().get_records(&name, &RRTypes::A) {
                    Some(records) => {
                        for record in records.iter().take(3) {
                            event.add_answer(&tld, record.clone());
                        }
                    }
                    None => return Err(io::Error::new(io::ErrorKind::Other, "Document not found"))
                }
            }
        }

        Ok(())
    }
}

