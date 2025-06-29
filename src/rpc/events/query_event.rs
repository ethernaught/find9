use rlibdns::messages::dns_query::DnsQuery;
use rlibdns::records::inter::record_base::RecordBase;
use rlibdns::utils::ordered_map::OrderedMap;
use crate::rpc::events::inter::event::Event;

pub struct QueryEvent {
    prevent_default: bool,
    query: DnsQuery,
    authoritative: bool,
    answers: OrderedMap<String, Vec<Box<dyn RecordBase>>>,
    authority_records: OrderedMap<String, Vec<Box<dyn RecordBase>>>,
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
            authority_records: OrderedMap::new(),
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

    pub fn calculate_total_answers(&self) -> usize {
        self.answers.values().map(|v| v.len()).sum()
    }

    pub fn has_authority_records(&self) -> bool {
        self.authority_records.len() > 0
    }

    pub fn add_authority_record(&mut self, query: &str, record: Box<dyn RecordBase>) {
        if self.authority_records.contains_key(&query.to_string()) {
            self.authority_records.get_mut(&query.to_string()).unwrap().push(record);
            return;
        }

        self.authority_records.insert(query.to_string(), vec![record]);
    }

    pub fn get_authority_records(&self) -> &OrderedMap<String, Vec<Box<dyn RecordBase>>> {
        &self.authority_records
    }

    pub fn get_authority_records_mut(&mut self) -> &mut OrderedMap<String, Vec<Box<dyn RecordBase>>> {
        &mut self.authority_records
    }

    pub fn calculate_total_authority_records(&self) -> usize {
        self.authority_records.values().map(|v| v.len()).sum()
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

    pub fn calculate_total_additional_records(&self) -> usize {
        self.additional_records.values().map(|v| v.len()).sum()
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
