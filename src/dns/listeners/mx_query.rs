use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::cname_record::CNameRecord;
use rlibdns::records::mx_record::MxRecord;
use rlibdns::records::ns_record::NsRecord;
use rlibdns::zone::zone::Zone;
use crate::dns::dns::ResponseResult;
use crate::MAX_ANSWERS;
use crate::rpc::events::request_event::RequestEvent;
use crate::utils::query_utils::{add_glue, chain_cname};

pub fn on_mx_query(zones: &Arc<RwLock<Zone>>) -> impl Fn(&mut RequestEvent) -> ResponseResult<()> {
    let zones = zones.clone();

    move |event| {
        let name = event.get_query().get_name().to_string();

        match zones.read().unwrap().get_deepest_zone(&name) {
            Some(zone) => {
                match zone.get_records(&RRTypes::CName) {
                    Some(records) => {
                        event.set_authoritative(zone.is_authority());

                        let record = records.first().unwrap();
                        event.add_answer(&name, record.clone());
                        let target = chain_cname(&zones, event, &record.as_any().downcast_ref::<CNameRecord>().unwrap().get_target().unwrap(), 0)?;

                        match zones.read().unwrap().get_deepest_zone(&target) {
                            Some(zone) => {
                                match zone.get_records(&event.get_query().get_type()) {
                                    Some(records) => {
                                        for record in records.iter().take(MAX_ANSWERS) {
                                            event.add_answer(&target, record.clone());
                                            add_glue(&zones, event, &record.as_any().downcast_ref::<MxRecord>().unwrap().get_server().unwrap());
                                        }
                                    }
                                    None => {
                                        match zone.get_records(&RRTypes::Ns) {
                                            Some(records) => {
                                                for record in records.iter().take(MAX_ANSWERS) {
                                                    event.add_authority_record(&target, record.clone());
                                                }
                                            }
                                            None => {}
                                        }
                                    }
                                }
                            }
                            None => {}
                        }
                    }
                    None => {
                        match zone.get_records(&event.get_query().get_type()) {
                            Some(records) => {
                                event.set_authoritative(zone.is_authority());

                                for record in records.iter().take(MAX_ANSWERS) {
                                    event.add_answer(&name, record.clone());
                                    add_glue(&zones, event, &record.as_any().downcast_ref::<MxRecord>().unwrap().get_server().unwrap());
                                }
                            }
                            None => {
                                match zone.get_records(&RRTypes::Ns) {
                                    Some(records) => {
                                        for record in records.iter().take(MAX_ANSWERS) {
                                            event.add_authority_record(&name, record.clone());
                                            add_glue(&zones, event, &record.as_any().downcast_ref::<NsRecord>().unwrap().get_server().unwrap());
                                        }
                                    }
                                    None => return Err(ResponseCodes::Refused)
                                }
                            }
                        }
                    }
                }
            }
            None => {
                match zones.read().unwrap().get_deepest_zone_with_records(&name, &RRTypes::Soa) {
                    Some((name, zone)) => {
                        event.set_authoritative(zone.is_authority());
                        event.add_authority_record(&name, zone.get_records(&RRTypes::Soa)
                            .ok_or(ResponseCodes::Refused)?.first().unwrap().clone());
                    }
                    None => return Err(ResponseCodes::Refused)
                }

                return Err(ResponseCodes::NxDomain);
            }
        }

        Ok(())
    }
}
