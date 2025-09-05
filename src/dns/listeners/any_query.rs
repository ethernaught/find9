use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::inter::rr_classes::RRClasses;
use rlibdns::records::hinfo_record::HInfoRecord;
use rlibdns::records::inter::record_base::RecordBase;
use rlibdns::zone::zone::Zone;
use crate::dns::dns::ResponseResult;
use crate::{ANY_QUERY_ALLOWED, MAX_ANSWERS};
use crate::rpc::events::query_event::QueryEvent;

pub fn on_any_query(zones: &Arc<RwLock<Zone>>) -> impl Fn(&mut QueryEvent) -> ResponseResult<()> {
    let zones = zones.clone();

    move |event| {
        let name = event.get_query().get_name();

        match zones.read().unwrap().get_deepest_zone(&name) {
            Some(zone) => {
                event.set_authoritative(zone.is_authority());

                if ANY_QUERY_ALLOWED {
                    for (_type, records) in zone.get_all_records() {
                        for record in records.iter().take(MAX_ANSWERS) {
                            event.add_answer(&name, record.clone());
                        }
                    }

                    return Ok(());
                }

                //;; OPT PSEUDOSECTION:
                // ; EDNS: version: 0, flags:; udp: 65535
                // ; EDE: 21 (Not Supported): (Type ANY Queries not supported here, RFC8482)

                let mut hinfo = HInfoRecord::new(3600, RRClasses::In);
                hinfo.set_cpu("RFC8482");
                hinfo.set_os("");

                event.add_answer(&name, hinfo.upcast());
            }
            None => return Err(ResponseCodes::Refused)
        }

        Ok(())
    }
}
