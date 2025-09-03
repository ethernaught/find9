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

                        //GET JOURNALS
                        //CONVERT JOURNALS TO BYTES
                        //THIS WOULD ENTAIL TAKING THE SOA AND GENERATING OTHER SOA WITH THE SERIAL_0 & SERIAL_1

                        //WE NEED THE AUTHORITATIVE RECORD FROM THE REQUEST TO GET THE IXFR NUMBER
                        //IF IXFR=2 then WE NEED TO START FROM JOURNAL 2 -> END

                        //EXAMPLE OF RESPONSE
                        //SOA(2)
                        // ... deletes for 2→3 ...
                        // SOA(3)
                        // ... adds for 2→3 ...
                        // SOA(3)
                        // ... deletes for 3→4 ...
                        // SOA(4)
                        // ... adds for 3→4 ...
                        // SOA(4)
                        // ... deletes for 4→5 ...
                        // SOA(5)
                        // ... adds for 4→5 ...


                        /*
                        let record = records.first().unwrap();
                        event.add_answer(&name, record.clone());

                        for (n, records) in zone.get_all_records_recursive().drain() {
                            for record in records {
                                event.add_answer(&format!("{n}{name}"), record.clone());
                            }
                        }

                        event.add_answer(&name, record.clone());
                        */
                    }
                    None => {}
                }
            }
            None => return Err(ResponseCodes::Refused) //KILL CONNECTION
        }

        Ok(())
    }
}
