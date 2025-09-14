use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::cname_record::CNameRecord;
use rlibdns::zone::zone_store::ZoneStore;
use crate::{MAX_ANSWERS, MAX_CNAME_CHAIN_SIZE};
use crate::dns::dns::ResponseResult;
use crate::rpc::events::request_event::RequestEvent;

pub fn chain_cname(store: &Arc<RwLock<ZoneStore>>, event: &mut RequestEvent, name: &str, depth: u8) -> ResponseResult<String> {
    match store.read().unwrap().get_deepest_zone(&name) {
        Some(zone) => {
            match zone.get_records(&RRTypes::CName) {
                Some(records) => {
                    if depth+1 >= MAX_CNAME_CHAIN_SIZE {
                        return Err(ResponseCodes::ServFail);
                    }

                    let record = records.get(0).unwrap();
                    event.add_answer(&name, record.clone());
                    chain_cname(store, event, &record.as_any().downcast_ref::<CNameRecord>().unwrap().get_target().unwrap(), depth+1)
                }
                None => Ok(name.to_string())
            }
        }
        None => {
            match store.read().unwrap().get_deepest_zone_with_name(&name) {
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

pub fn add_glue(store: &Arc<RwLock<ZoneStore>>, event: &mut RequestEvent, name: &str) {
    match store.read().unwrap().get_deepest_zone_with_name(&name) {
        Some((name, zone)) => {
            match zone.get_records(&RRTypes::A) {
                Some(records) => {
                    event.add_additional_record(&name, records.first().unwrap().clone());
                }
                None => {}
            }
        }
        None => {}
    }

    match store.read().unwrap().get_deepest_zone_with_name(&name) {
        Some((name, zone)) => {
            match zone.get_records(&RRTypes::Aaaa) {
                Some(records) => {
                    event.add_additional_record(&name, records.first().unwrap().clone());
                }
                None => {}
            }
        }
        None => {}
    }
}
