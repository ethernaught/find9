use std::io;
use rlibdns::records::soa_record::SoaRecord;
use crate::database::sqlite::Database;
use crate::rpc::events::inter::dns_query_event::DnsQueryEvent;
use crate::rpc::events::query_event::QueryEvent;

pub fn on_soa_query(database: &Database) -> impl Fn(&mut QueryEvent) -> io::Result<()> {
    let database = database.clone();
    move |event| {
        //let is_bogon = is_bogon(message.get_origin().unwrap());//if  { "network < 2" } else { "network > 0" };
        let is_bogon = "network < 2";
        let name = event.get_query().get_name().to_lowercase();
        let class = event.get_query().get_dns_class();

        let records = database.get(
            "soa",
            Some(vec!["class", "ttl", "content", "network"]),
            Some(format!("class = {} AND name = '{}' AND {}", class.get_code(), name, is_bogon).as_str()),
        );

        if records.is_empty() {
            return Err(io::Error::new(io::ErrorKind::Other, "Document not found"));
        }

        for record in records {
            let ttl = record.get("ttl").unwrap().parse::<u32>().unwrap();
            //let content = record.get("content").unwrap().as_str();

            //event.add_answer(&name, Box::new(SoaRecord::new(class, false, ttl, vec![content.to_string()])));
        }

        Ok(())
    }
}
