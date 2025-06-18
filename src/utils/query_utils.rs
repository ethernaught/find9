use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::cname_record::CNameRecord;
use crate::{MAX_ANSWERS, MAX_CNAME_CHAIN_SIZE};
use crate::dns::dns::ResponseResult;
use crate::rpc::events::query_event::QueryEvent;
use crate::zone::zone::Zone;

pub fn chain_cname(zones: &Arc<RwLock<Zone>>, event: &mut QueryEvent, name: &str, depth: u8) -> ResponseResult<String> {
    match zones.read().unwrap().get_deepest_zone(&name) {
        Some(zone) => {
            match zone.get_records(&RRTypes::CName) {
                Some(records) => {
                    if depth+1 < MAX_CNAME_CHAIN_SIZE {
                        let record = records.get(0).unwrap();
                        event.add_answer(&name, record.clone());
                        chain_cname(zones, event, &record.as_any().downcast_ref::<CNameRecord>().unwrap().get_target().unwrap(), depth+1)

                    } else {
                        Err(ResponseCodes::ServFail)
                    }
                }
                None => Ok(name.to_string())
            }
        }
        None => {
            match zones.read().unwrap().get_deepest_zone_with_records(&event.get_query().get_name(), &RRTypes::Soa) {
                Some((name, zone)) => {
                    for record in zone.get_records(&RRTypes::Soa)
                            .ok_or(ResponseCodes::Refused)?.iter().take(MAX_ANSWERS) {
                        event.add_authority_record(&name, record.clone());
                    }
                }
                None => return Err(ResponseCodes::Refused)
            }

            Err(ResponseCodes::NxDomain)
        }
    }
}
