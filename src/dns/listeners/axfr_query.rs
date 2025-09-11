use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::zone::zone::Zone;
use crate::dns::dns::ResponseResult;
use crate::rpc::events::request_event::RequestEvent;

pub fn on_axfr_query(zones: &Arc<RwLock<Zone>>) -> impl Fn(&mut RequestEvent) -> ResponseResult<()> {
    let zones = zones.clone();

    move |event| {
        let name = event.get_query().get_name().to_string();

        match zones.read().unwrap().get_deepest_zone(&name) {
            Some(zone) => {
                event.set_authoritative(zone.is_authority());

                match zone.get_records(&RRTypes::Soa) {
                    Some(records) => {
                        let record = records.first().unwrap();
                        event.add_answer(&name, record.clone());

                        for (n, records) in zone.get_all_records_recursive() {
                            for record in records {
                                event.add_answer(&format!("{n}{name}"), record.clone());

                                //for i in 0..26 {
                                //    event.add_answer(&format!("{n}{name}"), record.clone());
                                //}
                            }
                        }

                        event.add_answer(&name, record.clone());
                    }
                    None => {}
                }
            }
            None => return Err(ResponseCodes::Refused) //KILL CONNECTION
        }

        Ok(())
    }
}
