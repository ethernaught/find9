use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::cname_record::CNameRecord;
use crate::dns::dns::ResponseResult;
use crate::MAX_ANSWERS;
use crate::rpc::events::query_event::QueryEvent;
use crate::utils::query_utils::chain_cname;
use crate::zone::zone::Zone;

pub fn on_txt_query(zones: &Arc<RwLock<Zone>>) -> impl Fn(&mut QueryEvent) -> ResponseResult<()> {
    let zones = zones.clone();

    move |event| {
        match zones.read().unwrap().get_deepest_zone(&event.get_query().get_name()) {
            Some(zone) => {
                event.set_authoritative(true);

                match zone.get_records(&RRTypes::CName) {
                    Some(records) => {
                        let record = records.get(0).unwrap();
                        event.add_answer(&event.get_query().get_name(), record.clone());
                        chain_cname(&zones, event, &record.as_any().downcast_ref::<CNameRecord>().unwrap().get_target().unwrap(), 0)?;
                    }
                    None => {
                        match zone.get_records(&event.get_query().get_type()) {
                            Some(records) => {
                                for record in records.iter().take(MAX_ANSWERS) {
                                    event.add_answer(&event.get_query().get_name(), record.clone());
                                }
                            }
                            None => return Err(ResponseCodes::NxDomain)
                        }
                    }
                }
            }
            None => event.set_authoritative(false)
        }

        Ok(())
    }
}
