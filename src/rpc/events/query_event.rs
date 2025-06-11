use rlibdns::messages::dns_query::DnsQuery;
use rlibdns::records::inter::record_base::RecordBase;
use rlibdns::utils::ordered_map::OrderedMap;
use crate::rpc::events::inter::event::Event;

pub struct QueryEvent {
    prevent_default: bool,
    query: DnsQuery,
    authoritative: bool,
    answers: OrderedMap<String, Vec<Box<dyn RecordBase>>>,
    name_servers: OrderedMap<String, Vec<Box<dyn RecordBase>>>,
    additional_records: OrderedMap<String, Vec<Box<dyn RecordBase>>>,
    received_time: u128
}

impl QueryEvent {

    pub fn new(query: DnsQuery) -> Self {
        Self {
            prevent_default: false,
            query,
            authoritative: false,
            answers: OrderedMap::new(),
            name_servers: OrderedMap::new(),
            additional_records: OrderedMap::new(),
            received_time: 0
        }
    }

    pub fn set_authoritative(&mut self, authoritative: bool) {
        self.authoritative = authoritative;
    }

    pub fn is_authoritative(&self) -> bool {
        self.authoritative
    }

    pub fn get_query(&self) -> &DnsQuery {
        &self.query
    }

    pub fn has_answers(&self) -> bool {
        self.answers.len() > 0
    }

    pub fn add_answer(&mut self, query: &str, record: Box<dyn RecordBase>) {
        if self.answers.contains_key(&query.to_string()) {
            self.answers.get_mut(&query.to_string()).unwrap().push(record);
            return;
        }

        self.answers.insert(query.to_string(), vec![record]);
    }

    pub fn get_answers(&self) -> &OrderedMap<String, Vec<Box<dyn RecordBase>>> {
        &self.answers
    }

    pub fn get_answers_mut(&mut self) -> &mut OrderedMap<String, Vec<Box<dyn RecordBase>>> {
        &mut self.answers
    }

    pub fn has_name_servers(&self) -> bool {
        self.name_servers.len() > 0
    }

    pub fn add_name_server(&mut self, query: &str, record: Box<dyn RecordBase>) {
        if self.name_servers.contains_key(&query.to_string()) {
            self.name_servers.get_mut(&query.to_string()).unwrap().push(record);
            return;
        }

        self.name_servers.insert(query.to_string(), vec![record]);
    }

    pub fn get_name_servers(&self) -> &OrderedMap<String, Vec<Box<dyn RecordBase>>> {
        &self.name_servers
    }

    pub fn get_name_servers_mut(&mut self) -> &mut OrderedMap<String, Vec<Box<dyn RecordBase>>> {
        &mut self.name_servers
    }

    pub fn has_additional_records(&self) -> bool {
        self.additional_records.len() > 0
    }

    pub fn add_additional_record(&mut self, query: &str, record: Box<dyn RecordBase>) {
        if self.additional_records.contains_key(&query.to_string()) {
            self.additional_records.get_mut(&query.to_string()).unwrap().push(record);
            return;
        }

        self.additional_records.insert(query.to_string(), vec![record]);
    }

    pub fn get_additional_records(&self) -> &OrderedMap<String, Vec<Box<dyn RecordBase>>> {
        &self.additional_records
    }

    pub fn get_additional_records_mut(&mut self) -> &mut OrderedMap<String, Vec<Box<dyn RecordBase>>> {
        &mut self.additional_records
    }
}

impl Event for QueryEvent {

    fn is_prevent_default(&self) -> bool {
        self.prevent_default
    }

    fn prevent_default(&mut self) {
        self.prevent_default = true;
    }
}
