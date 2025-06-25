use std::sync::{Arc, RwLock};
use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::cname_record::CNameRecord;
use rlibdns::records::ns_record::NsRecord;
use crate::dns::dns::ResponseResult;
use crate::MAX_ANSWERS;
use crate::rpc::events::query_event::QueryEvent;
use crate::utils::query_utils::{add_glue, chain_cname};
use crate::zone::zone::Zone;

pub fn on_any_query(zones: &Arc<RwLock<Zone>>) -> impl Fn(&mut QueryEvent) -> ResponseResult<()> {
    let zones = zones.clone();

    move |event| {
        let name = event.get_query().get_name();

        Ok(())
    }
}
