use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::cname_record::CNameRecord;
use crate::{MAX_ANSWERS, MAX_CNAME_CHAIN_SIZE};
use crate::dns::dns::ResponseResult;
use crate::rpc::events::query_event::QueryEvent;
use crate::zone::zone::Zone;

pub fn chain_cname(zones: &Arc<RwLock<Zone>>, event: &mut QueryEvent, name: &str, depth: u8) -> ResponseResult<()> {
    let zones_lock = zones.read().unwrap();

    match zones_lock.get_deepest_zone(&name).ok_or(ResponseCodes::NxDomain)?.get_records(&RRTypes::CName) {
        Some(records) => {
            if depth+1 < MAX_CNAME_CHAIN_SIZE {
                let record = records.get(0).unwrap();
                event.add_answer(&name, record.clone());
                chain_cname(zones, event, &record.as_any().downcast_ref::<CNameRecord>().unwrap().get_target().unwrap(), depth+1)?;

            } else {
                return Err(ResponseCodes::ServFail) //EXESSIVE CHAIN
            }
        }
        None => {
            match zones_lock.get_deepest_zone(&name).unwrap().get_records(&event.get_query().get_type()) {
                Some(records) => {
                    for record in records.iter().take(MAX_ANSWERS) {
                        event.add_answer(&name, record.clone());
                    }
                }
                None => return Err(ResponseCodes::NxDomain) //CHAIN FAIL
            }
        }
    }

    Ok(())
}
