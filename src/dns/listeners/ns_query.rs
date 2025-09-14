use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::cname_record::CNameRecord;
use rlibdns::records::ns_record::NsRecord;
use rlibdns::zone::zone_store::ZoneStore;
use crate::dns::dns::ResponseResult;
use crate::MAX_ANSWERS;
use crate::rpc::events::request_event::RequestEvent;
use crate::utils::query_utils::add_glue;

pub fn on_ns_query(store: &Arc<RwLock<ZoneStore>>) -> impl Fn(&mut RequestEvent) -> ResponseResult<()> {
    let store = store.clone();

    move |event| {
        let name = event.get_query().get_name().to_string();

        match store.read().unwrap().get_zone_exact(&name) {
            Some(zone) => {
                match zone.get_records(&RRTypes::CName) {
                    Some(records) => {
                        let record = records.first().unwrap();
                        event.add_answer(&name, record.clone());
                        let target = record.as_any().downcast_ref::<CNameRecord>().unwrap().get_target().unwrap();

                        event.set_authoritative(zone.is_authority());

                        match store.read().unwrap().get_deepest_zone(&target) {
                            Some(zone) => {
                                match zone.get_records(&event.get_query().get_type()) {
                                    Some(records) => {
                                        for record in records.iter().take(MAX_ANSWERS) {
                                            event.add_authority_record(&target, record.clone());
                                        }
                                    }
                                    None => {}
                                }
                            }
                            None => {}
                        }
                    }
                    None => {
                        match zone.get_records(&event.get_query().get_type()) {
                            Some(records) => {
                                if zone.is_authority() {
                                    event.set_authoritative(true);

                                    for record in records.iter().take(MAX_ANSWERS) {
                                        event.add_answer(&name, record.clone());
                                    }

                                } else {
                                    for record in records.iter().take(MAX_ANSWERS) {
                                        event.add_authority_record(&name, record.clone());
                                        add_glue(&store, event, &record.as_any().downcast_ref::<NsRecord>().unwrap().get_server().unwrap());
                                    }
                                }
                            }
                            None => return Err(ResponseCodes::Refused)
                        }
                    }
                }
            }
            None => {
                return match store.read().unwrap().get_deepest_zone_with_name(&name) {
                    Some((name, zone)) => {
                        event.set_authoritative(zone.is_authority());
                        event.add_authority_record(&name, zone.get_records(&RRTypes::Soa)
                            .ok_or(ResponseCodes::Refused)?.first().unwrap().clone());
                        Err(ResponseCodes::NxDomain)
                    }
                    None => Err(ResponseCodes::Refused)
                }
            }
        }

        Ok(())
    }
}
