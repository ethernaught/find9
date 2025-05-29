use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::cname_record::CNameRecord;
use crate::MAX_CNAME_CHAIN_SIZE;
use crate::rpc::events::query_event::QueryEvent;
use crate::utils::domain_utils::split_domain;
use crate::zone::zone::Zone;

pub fn chain_cname(zones: &Arc<RwLock<HashMap<String, Zone>>>, event: &mut QueryEvent, record: &CNameRecord, depth: u8) {
    let target = record.get_target().unwrap();
    let (name, tld) = split_domain(&target).unwrap();

    match zones.read().unwrap().get(&tld).unwrap().get_records(&name, &RRTypes::CName) {
        Some(records) => {
            if depth+1 < MAX_CNAME_CHAIN_SIZE {
                let record = records.get(0).unwrap();
                event.add_answer(&target, record.clone());
                chain_cname(zones, event, record.as_any().downcast_ref::<CNameRecord>().unwrap(), depth+1);

            } else {
                event.add_answer(&target, records.get(0).unwrap().clone());
            }
        }
        None => {}
    }

    match zones.read().unwrap().get(&tld).unwrap().get_records(&name, &event.get_query().get_type()) {
        Some(records) => {
            for record in records.iter().take(3) {
                event.add_answer(&target, record.clone());
            }
        }
        None => {}
    }
}
