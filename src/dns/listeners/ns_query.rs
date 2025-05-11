use std::io;
use crate::database::sqlite::Database;
use crate::rpc::events::inter::dns_query_event::DnsQueryEvent;
use crate::rpc::events::query_event::QueryEvent;

pub fn on_ns_query(database: &Database) -> impl Fn(&mut QueryEvent) -> io::Result<()> {
    let database = database.clone();
    move |event| {
        //let is_bogon = is_bogon(message.get_origin().unwrap());//if  { "network < 2" } else { "network > 0" };
        let is_bogon = "network < 2";
        let name = event.get_query().get_name().to_lowercase();
        let class = event.get_query().get_dns_class();

        let records = database.get(
            "cname",
            Some(vec!["class", "ttl", "target", "network"]),
            Some(format!("class = {} AND name = '{}' AND {}", class.get_code(), name, is_bogon).as_str()),
        );

        Ok(())
    }
}
