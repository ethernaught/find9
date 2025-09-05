use std::sync::{Arc, RwLock};
use rlibdns::journal::inter::txn_op_codes::TxnOpCodes;
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::inter::record_base::RecordBase;
use rlibdns::records::soa_record::SoaRecord;
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

                        //ADD CONDENSING...
                        //Client has serial 10, server is at 13. Valid IXFR replies include:
                        //
                        // Three deltas: (10→11), then (11→12), then (12→13), oldest first.
                        // IETF Datatracker
                        //
                        // One condensed delta: (10→13) with all necessary deletes/adds to transform 10 into 13.

                        //WE ARE NOT ACCURATE TO BIND9 WE NEED TO VERIFY WE ARE PARSING JOURNAL AND IF JOURNAL MATCHES RESPONSE GIVEN BY IXFR

                        let query_serial = 2;

                        let record = records.first().unwrap().as_any().downcast_ref::<SoaRecord>().unwrap();

                        //DETERMINE LAST SOA RECORD NUMBER ADD TO BEGINNING AND END...
                        event.add_answer(&name, record.clone().upcast());


                        for (_, txn) in zone.get_txn_from(query_serial) {
                            let mut first_record = record.clone();
                            first_record.set_serial(txn.get_serial_0());
                            event.add_answer(&name, first_record.upcast());

                            //DELETES
                            for (name, record) in txn.get_records(TxnOpCodes::Delete) {
                                event.add_answer(name, record.clone());
                            }

                            let mut second_record = record.clone();
                            second_record.set_serial(txn.get_serial_1());
                            event.add_answer(&name, second_record.upcast());

                            //ADDS
                            for (name, record) in txn.get_records(TxnOpCodes::Add) {
                                event.add_answer(name, record.clone());
                            }
                        }

                        event.add_answer(&name, record.clone().upcast());




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
