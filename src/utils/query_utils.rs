use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::cname_record::CNameRecord;
use crate::{MAX_ANSWERS, MAX_CNAME_CHAIN_SIZE};
use crate::rpc::events::query_event::QueryEvent;
use crate::utils::domain_utils::split_domain;
use crate::zone::zone::Zone;

pub fn chain_cname(zones: &Arc<RwLock<HashMap<String, Zone>>>, event: &mut QueryEvent, record: &CNameRecord, depth: u8) {
    let target = record.get_target().unwrap();
    //let (name, tld) = match split_domain(&event.get_query().get_name()) {
    //    Some((name, tld)) => (name, tld),
    //    None => return
    //};
    let (name, tld) = split_domain(&target).unwrap();

    /*
    SERVER CONTAINS
    x1 -> x2 -> x3 -> ANSWERS (it shouldnt show all cnames just the answer as if the cname was the answer)

    ;; QUESTION SECTION:
    ;x1.find9.net.			IN	A

    ;; ANSWER SECTION:
    x1.find9.net.		300	IN	A	104.21.42.137
    x1.find9.net.		300	IN	A	172.67.206.28
    */

    //match zones.read().unwrap().get(&tld).unwrap().get_records(&name, &RRTypes::CName) {
    //    Some(records) => {
    //        if depth+1 < MAX_CNAME_CHAIN_SIZE {
    //            let record = records.get(0).unwrap();
    //            event.add_answer(&target, record.clone());
    //            chain_cname(zones, event, record.as_any().downcast_ref::<CNameRecord>().unwrap(), depth+1);

    //        } else {
    //            event.add_answer(&target, records.get(0).unwrap().clone());
    //        }
    //    }
    //    None => {}
    //}

    //match zones.read().unwrap().get(&tld).unwrap().get_records(&name, &event.get_query().get_type()) {
    //    Some(records) => {
    //        for record in records.iter().take(MAX_ANSWERS) {
    //            event.add_answer(&target, record.clone());
    //        }
    //    }
    //    None => {}
    //}

    /*
    if depth >= MAX_CNAME_CHAIN_SIZE {
        return;
    }
    */


    /*
    let zones = zones.lock().unwrap();
    let tld = get_tld(&name); // implement get_tld() if not already

    // 1. Try to resolve CNAME
    if let Some(cname_records) = zones.get(&tld).and_then(|z| z.get_records(&name, &RRTypes::CName)) {
        let cname = cname_records[0].clone();
        let cname_target = cname.as_any().downcast_ref::<CNameRecord>().unwrap().get_target().to_string();

        event.add_answer(&name, cname.clone());

        // 2. Recurse on CNAME target
        //follow_cname_chain(zones, event, cname_target, rr_type, depth + 1);
        chain_cname(zones, event, record.as_any().downcast_ref::<CNameRecord>().unwrap(), depth+1);

    } else {
        // 3. No CNAME → return actual answer (e.g., A, AAAA, etc.)
        if let Some(answer_records) = zones.get(&tld).and_then(|z| z.get_records(&name, &rr_type)) {
            for record in answer_records.iter().take(MAX_ANSWERS) {
                event.add_answer(&name, record.clone());
            }
        }
    }
    */
}
