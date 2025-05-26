use std::collections::HashMap;
use std::io;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::cname_record::CNameRecord;
use rlibdns::records::inter::record_base::RecordBase;
use rlibdns::zone::zone_parser::ZoneParser;
use crate::RecordMap;
use crate::rpc::events::query_event::QueryEvent;

pub fn on_a_query() -> impl Fn(&mut QueryEvent) -> io::Result<()> {
    let mut zones = RecordMap::new();

    let mut parser = ZoneParser::new("/home/brad/find9/find9.net.zone", "find9.net").unwrap();

    for (name, record) in parser.iter() {
        println!("{}: {:?}", name, record);

        zones
            .entry(name)
            .or_insert_with(HashMap::new)
            .entry(record.get_type())
            .or_insert_with(Vec::new)
            .push(record);
    }

    move |event| {
        match zones.get(&event.get_query().get_name()).unwrap().get(&RRTypes::CName) {
            Some(records) => {
                for record in records.iter().take(3) {
                    if let Some(record) = record.as_any().downcast_ref::<CNameRecord>() {
                        let target = record.get_target().unwrap();

                        match zones.get(&target).unwrap().get(&RRTypes::A) {
                            Some(records) => {
                                for record in records.iter().take(3) {
                                    event.add_answer(&target, record.clone());
                                }
                            }
                            None => return Err(io::Error::new(io::ErrorKind::Other, "Document not found"))
                        }
                    }
                }
            }
            None => {
                match zones.get(&event.get_query().get_name()).unwrap().get(&RRTypes::A) {
                    Some(records) => {
                        for record in records.iter().take(3) {
                            event.add_answer(&event.get_query().get_name(), record.clone());
                        }
                    }
                    None => return Err(io::Error::new(io::ErrorKind::Other, "Document not found"))
                }
            }
        }

        Ok(())
    }
}
