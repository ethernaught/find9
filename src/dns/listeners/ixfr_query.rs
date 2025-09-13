use std::sync::{Arc, RwLock};
use rlibdns::journal::inter::txn_op_codes::TxnOpCodes;
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::inter::record_base::RecordBase;
use rlibdns::records::soa_record::SoaRecord;
use rlibdns::zone::zone::Zone;
use crate::dns::dns::ResponseResult;
use crate::rpc::events::request_event::RequestEvent;

pub fn on_ixfr_query(zones: &Arc<RwLock<Zone>>) -> impl Fn(&mut RequestEvent) -> ResponseResult<()> {
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

                        //ADD CONDENSING...
                        //Client has serial 10, server is at 13. Valid IXFR replies include:
                        //
                        // Three deltas: (10→11), then (11→12), then (12→13), oldest first.
                        // IETF Datatracker
                        //
                        // One condensed delta: (10→13) with all necessary deletes/adds to transform 10 into 13.

                        let mut current_soa = record.as_any().downcast_ref::<SoaRecord>().unwrap().clone();
                        let current_serial = current_soa.get_serial();

                        let client_serial_opt = event
                            .get_request_authority_records()
                            .iter()
                            .find_map(|(q, r)| {
                                if q == &name {
                                    r.as_any()
                                        .downcast_ref::<SoaRecord>()
                                        .map(|soa| soa.get_serial())
                                } else {
                                    None
                                }
                            });

                        match client_serial_opt {
                            Some(client_serial) => {
                                if client_serial >= current_serial {
                                    return Ok(());
                                }

                                let mut it = zone.get_txns_from(client_serial).peekable();

                                match it.peek() {
                                    Some(_) => {
                                        for (_, txn) in it {
                                            current_soa.set_serial(txn.get_serial_0());
                                            event.add_answer(&name, current_soa.clone().upcast());

                                            //DELETES
                                            for (name, record) in txn.get_records(TxnOpCodes::Delete) {
                                                event.add_answer(name, record.clone());
                                            }

                                            current_soa.set_serial(txn.get_serial_1());
                                            event.add_answer(&name, current_soa.clone().upcast());

                                            //ADDS
                                            for (name, record) in txn.get_records(TxnOpCodes::Add) {
                                                event.add_answer(name, record.clone());
                                            }
                                        }
                                    }
                                    None => {
                                        //AXFR
                                        for (n, records) in zone.get_all_records_recursive() {
                                            for record in records {
                                                event.add_answer(&format!("{n}{name}"), record.clone());
                                            }
                                        }
                                    }
                                }
                            }
                            None => {
                                //AXFR
                                for (n, records) in zone.get_all_records_recursive() {
                                    for record in records {
                                        event.add_answer(&format!("{n}{name}"), record.clone());
                                    }
                                }
                            }
                        }

                        event.add_answer(&name, current_soa.upcast());
                    }
                    None => return Err(ResponseCodes::ServFail)
                }
            }
            None => return Err(ResponseCodes::Refused) //KILL CONNECTION
        }

        Ok(())
    }
}
