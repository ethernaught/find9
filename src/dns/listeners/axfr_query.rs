use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::inter::rr_types::RRTypes;
use crate::dns::dns::ResponseResult;
use crate::MAX_ANSWERS;
use crate::rpc::events::query_event::QueryEvent;
use crate::zone::zone::Zone;

pub fn on_axfr_query(zones: &Arc<RwLock<Zone>>) -> impl Fn(&mut QueryEvent) -> ResponseResult<()> {
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

                        //LOOP OVER RECORDS THEN ZONES AND ADD EVERYTHING RECURSIVELY

                        for (n, records) in zone.get_all_records_recursive().drain() {
                            for record in records {
                                if record.get_type().eq(&RRTypes::Soa) {
                                    continue;
                                }

                                println!("{}: {}", format!("{n}{name}"), record);
                                event.add_answer(&format!("{n}{name}"), record.clone());
                            }
                        }

                        //OUR PROBLEM IS THAT THE RECORDS GET SAVED INTO A HASHMAP
                        //THE HASHMAP IS STRING BASED, SO WE WILL NEED TO CHANGE THIS TO NOT
                        //USE A HASHMAP BUT A VECTOR INSTEAD...

                        //DONT FULL-FILL THE LAST ANSWER MAYBE SO THAT THE TCP CAN BREAK IT
                        //UP PROPERLY UNLESS WE WANT TO PASS PACKET-LIMIT INTO THIS FUNCTION...

                        event.add_answer(&name, record.clone());
                    }
                    None => {}
                }
                //ONLY ALLOW SPECIFIC IPS
                //SOA
                //OTHER RECORDS - RECURSIVE
                //SOA
            }
            None => return Err(ResponseCodes::Refused) //KILL CONNECTION
        }

        Ok(())
    }
}
