use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::inter::rr_types::RRTypes;
use crate::dns::dns::ResponseResult;
use crate::rpc::events::query_event::QueryEvent;
use crate::zone::zone::Zone;

pub fn on_ixfr_query(zones: &Arc<RwLock<Zone>>) -> impl Fn(&mut QueryEvent) -> ResponseResult<()> {
    let zones = zones.clone();

    move |event| {
        let name = event.get_query().get_name();

        match zones.read().unwrap().get_deepest_zone(&name) {
            Some(zone) => {
                event.set_authoritative(zone.is_authority());

                match zone.get_records(&RRTypes::Soa) {
                    Some(records) => {
                        let record = records.first().unwrap();
                        event.add_answer(&name, record.clone());

                        //VERIFY SOA VALIDITY FROM REQUEST SOMEHOW....

                        for (n, records) in zone.get_all_records_recursive().drain() {
                            for record in records {
                                if record.get_type().eq(&RRTypes::Soa) {
                                    continue;
                                }

                                event.add_answer(&format!("{n}{name}"), record.clone());
                            }
                        }

                        //DONT FULL-FILL THE LAST ANSWER MAYBE SO THAT THE TCP CAN BREAK IT
                        //UP PROPERLY UNLESS WE WANT TO PASS PACKET-LIMIT INTO THIS FUNCTION...
                        //then again maybe we can determine the byte size and do it that way...

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
