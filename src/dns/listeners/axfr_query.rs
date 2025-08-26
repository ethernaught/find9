use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::inter::rr_types::RRTypes;
use crate::dns::dns::ResponseResult;
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

                        //THIS BUG ONLY OCCURS WITH DUPED RECORDS IN SAME ORDER NOT LOOPING OVER X TIMES...
                        for (n, records) in zone.get_all_records_recursive().drain() {
                            for record in records {
                                //if record.get_type().eq(&RRTypes::Soa) {
                                //    continue;
                                //}

                                event.add_answer(&format!("{n}{name}"), record.clone());

                                println!("{}", format!("{n}{name}"));



                                //if !record.get_type().eq(&RRTypes::Https) && !record.get_type().eq(&RRTypes::Svcb) {
                                //if !record.get_type().eq(&RRTypes::CName) && !record.get_type().eq(&RRTypes::Https) {
                                //    continue;
                                //}

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
