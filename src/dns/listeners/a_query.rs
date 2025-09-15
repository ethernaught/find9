use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::cname_record::CNameRecord;
use rlibdns::records::ns_record::NsRecord;
use rlibdns::utils::fqdn_utils::fqdn_to_relative;
use rlibdns::zone::zone_store::ZoneStore;
use crate::dns::dns::ResponseResult;
use crate::MAX_ANSWERS;
use crate::rpc::events::request_event::RequestEvent;
use crate::utils::query_utils::{add_glue, chain_cname};

pub fn on_a_query(store: &Arc<RwLock<ZoneStore>>) -> impl Fn(&mut RequestEvent) -> ResponseResult<()> {
    let store = store.clone();

    move |event| {
        let name = event.get_query().get_fqdn().to_string();

        match store.read().unwrap().get_deepest_zone_with_name(&name) {
            Some((apex, zone)) => {
                let sub = fqdn_to_relative(&apex, &name).unwrap();
                match zone.get_records(&sub, &RRTypes::CName) {
                    Some(records) => {
                        let record = records.first().unwrap();
                        event.add_answer(&name, record.clone());
                        let target = chain_cname(&apex, zone, event, &record.as_any().downcast_ref::<CNameRecord>().unwrap().get_target().unwrap(), 0)?;

                        event.set_authoritative(zone.is_authority());

                        let sub = fqdn_to_relative(&apex, &target).unwrap();
                        match zone.get_records(&sub, &event.get_query().get_type()) {
                            Some(records) => {
                                for record in records.iter().take(MAX_ANSWERS) {
                                    event.add_answer(&target, record.clone());
                                }
                            }
                            None => {
                                match zone.get_records(&sub, &RRTypes::Ns) {
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
                    None => {
                        match zone.get_records(&sub, &event.get_query().get_type()) {
                            Some(records) => {
                                event.set_authoritative(zone.is_authority());

                                for record in records.iter().take(MAX_ANSWERS) {
                                    event.add_answer(&name, record.clone());
                                }
                            }
                            None => {
                                match zone.get_records(&sub, &RRTypes::Soa) {
                                    Some(records) => {
                                        event.set_authoritative(zone.is_authority());
                                        event.add_authority_record(&apex, records.first().unwrap().clone());
                                    }
                                    None => {
                                        match zone.get_records(&sub, &RRTypes::Ns) {
                                            Some(records) => {
                                                for record in records.iter().take(MAX_ANSWERS) {
                                                    event.add_authority_record(&name, record.clone());
                                                    add_glue(zone, &apex, event, &record.as_any().downcast_ref::<NsRecord>().unwrap().get_server().unwrap());
                                                }
                                            }
                                            None => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            None => {
                return match store.read().unwrap().get_deepest_zone_with_name(&name) {
                    Some((apex, zone)) => {
                        event.set_authoritative(zone.is_authority());
                        event.add_authority_record(&apex, zone.get_records("", &RRTypes::Soa)
                            .ok_or(ResponseCodes::Refused)?.first().unwrap().clone());
                        Err(ResponseCodes::NxDomain)
                    }
                    None => Err(ResponseCodes::Refused)
                };
            }
        }

        Ok(())
    }
}
