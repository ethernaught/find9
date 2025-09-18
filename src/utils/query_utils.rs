use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::cname_record::CNameRecord;
use rlibdns::records::inter::record_base::RecordBase;
use rlibdns::utils::fqdn_utils::fqdn_to_relative;
use rlibdns::zone::zone::Zone;
use rlibdns::zone::zone_store::ZoneStore;
use crate::{MAX_ANSWERS, MAX_CNAME_CHAIN_SIZE};
use crate::dns::dns::ResponseResult;
use crate::rpc::events::request_event::RequestEvent;

pub fn chain_cname(zone: &Zone, apex: &str, event: &mut RequestEvent, name: &str, depth: u8) -> ResponseResult<String> {
    let sub = fqdn_to_relative(apex, name).unwrap();

    match zone.get_records(&sub, &RRTypes::CName) {
        Some(records) => {
            if depth+1 >= MAX_CNAME_CHAIN_SIZE {
                return Err(ResponseCodes::ServFail);
            }

            let record = records.get(0).unwrap();
            event.add_answer(&name, record.clone());
            let response = chain_cname(zone, apex, event, &record.as_any().downcast_ref::<CNameRecord>().unwrap().get_target().unwrap(), depth+1)?;
            Ok(response)
        }
        None => Ok(name.to_string())
    }

    /*
    match store.read().unwrap().get_zone_exact(&name) {
        Some(zone) => {
            match zone.get_records(&RRTypes::CName) {
                Some(records) => {
                    if depth+1 >= MAX_CNAME_CHAIN_SIZE {
                        return Err(ResponseCodes::ServFail);
                    }

                    let record = records.get(0).unwrap();
                    let response = chain_cname(store, event, &record.as_any().downcast_ref::<CNameRecord>().unwrap().get_target().unwrap(), depth+1)?;
                    event.add_answer(&name, record.clone());
                    Ok(response)
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
    }*/
    //Ok("".to_string())
}

pub fn add_glue(zone: &Zone, apex: &str, event: &mut RequestEvent, name: &str) {
    let sub = fqdn_to_relative(apex, name).unwrap();

    match zone.get_records(&sub, &RRTypes::A) {
        Some(records) => {
            event.add_additional_record(&name, records.first().unwrap().clone());
        }
        None => {}
    }

    match zone.get_records(&sub, &RRTypes::Aaaa) {
        Some(records) => {
            event.add_additional_record(&name, records.first().unwrap().clone());
        }
        None => {}
    }
    /*
    match store.read().unwrap().get_zone_exact(&name) {
        Some(zone) => {
            match zone.get_records(&RRTypes::A) {
                Some(records) => {
                    event.add_additional_record(&name, records.first().unwrap().clone());
                }
                None => {}
            }
        }
        None => {}
    }

    match store.read().unwrap().get_zone_exact(&name) {
        Some(zone) => {
            match zone.get_records(&RRTypes::Aaaa) {
                Some(records) => {
                    event.add_additional_record(&name, records.first().unwrap().clone());
                }
                None => {}
            }
        }
        None => {}
    }*/
}
