use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::response_codes::ResponseCodes;
use crate::dns::dns::ResponseResult;
use crate::MAX_ANSWERS;
use crate::rpc::events::query_event::QueryEvent;
use crate::zone::zone::Zone;

pub fn on_soa_query(zones: &Arc<RwLock<Zone>>) -> impl Fn(&mut QueryEvent) -> ResponseResult<()> {
    let zones = zones.clone();

    move |event| {
        let name = event.get_query().get_name();

        match zones.read().unwrap().get_deepest_zone(&name) {
            Some(zone) => {
                event.set_authoritative(zone.is_authority());

                match zone.get_records(&event.get_query().get_type()) {
                    Some(records) => {
                        for record in records.iter().take(MAX_ANSWERS) {
                            event.add_answer(&name, record.clone());
                        }

                        return Ok(());
                    }
                    None => {}
                }
            }
            None => {
                event.set_authoritative(false)
            }
        }
        
        match zones.read().unwrap().get_deepest_records(&name, &event.get_query().get_type()) {
            Some((n, records)) => {
                for record in records.iter().take(MAX_ANSWERS) {
                    event.add_name_server(&n, record.clone());
                }

                Ok(())
            }
            None => Err(ResponseCodes::Refused)
        }
    }
}
