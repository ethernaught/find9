use std::collections::HashMap;
use std::io;
use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::cname_record::CNameRecord;
use crate::MAX_ANSWERS;
use crate::rpc::events::query_event::QueryEvent;
use crate::utils::query_utils::chain_cname;
use crate::zone::zone::Zone;

pub fn on_aaaa_query(zones: &Arc<RwLock<HashMap<String, Zone>>>) -> impl Fn(&mut QueryEvent) -> io::Result<()> {
    let zones = zones.clone();

    move |event| {
        /*
        let (name, tld) = match split_domain(&event.get_query().get_name()) {
            Some((name, tld)) => (name, tld),
            None => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid query name"))
        };

        match zones.read().unwrap().get(&tld) {
            Some(zone) => {
                event.set_authoritative(true);

                match zone.get_records(&name, &RRTypes::CName) {
                    Some(records) => {
                        let record = records.get(0).unwrap();
                        event.add_answer(&event.get_query().get_name(), record.clone());
                        chain_cname(&zones, event, record.as_any().downcast_ref::<CNameRecord>().unwrap(), 0);
                    }
                    None => {
                        match zone.get_records(&name, &event.get_query().get_type()) {
                            Some(records) => {
                                event.set_authoritative(true);

                                for record in records.iter().take(MAX_ANSWERS) {
                                    event.add_answer(&tld, record.clone());
                                }
                            }
                            None => return Err(io::Error::new(io::ErrorKind::Other, "Document not found"))
                        }
                    }
                }
            }
            None => event.set_authoritative(false)
        }
        */

        Ok(())
    }
}
