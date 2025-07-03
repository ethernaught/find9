use rlibdns::messages::dns_query::DnsQuery;
use rlibdns::records::inter::record_base::RecordBase;
use crate::rpc::events::inter::event::Event;

pub struct QueryEvent {
    prevent_default: bool,
    query: DnsQuery,
    authoritative: bool,
    answers: Vec<(String, Box<dyn RecordBase>)>, //WE HAVE TO SWITCH FROM ORDERED_MAP TO VEC FOR RFC 5936
    authority_records: Vec<(String, Box<dyn RecordBase>)>,
    additional_records: Vec<(String, Box<dyn RecordBase>)>,
    received_time: u128
}

impl QueryEvent {

    pub fn new(query: DnsQuery) -> Self {
        Self {
            prevent_default: false,
            query,
            authoritative: false,
            answers: Vec::new(),
            authority_records: Vec::new(),
            additional_records: Vec::new(),
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
        !self.answers.is_empty()
    }

    pub fn add_answer(&mut self, query: &str, record: Box<dyn RecordBase>) {
        self.answers.push((query.to_string(), record));
    }

    pub fn get_answers(&self) -> &Vec<(String, Box<dyn RecordBase>)> {
        &self.answers
    }

    pub fn get_answers_mut(&mut self) -> &mut Vec<(String, Box<dyn RecordBase>)> {
        &mut self.answers
    }

    pub fn calculate_total_answers(&self) -> usize {
        self.answers.len()
    }

    pub fn has_authority_records(&self) -> bool {
        !self.authority_records.is_empty()
    }

    pub fn add_authority_record(&mut self, query: &str, record: Box<dyn RecordBase>) {
        self.authority_records.push((query.to_string(), record));
    }

    pub fn get_authority_records(&self) -> &Vec<(String, Box<dyn RecordBase>)> {
        &self.authority_records
    }

    pub fn get_authority_records_mut(&mut self) -> &mut Vec<(String, Box<dyn RecordBase>)> {
        &mut self.authority_records
    }

    pub fn calculate_total_authority_records(&self) -> usize {
        self.authority_records.len()
    }

    pub fn has_additional_records(&self) -> bool {
        !self.additional_records.is_empty()
    }

    pub fn add_additional_record(&mut self, query: &str, record: Box<dyn RecordBase>) {
        self.additional_records.push((query.to_string(), record));
    }

    pub fn get_additional_records(&self) -> &Vec<(String, Box<dyn RecordBase>)> {
        &self.additional_records
    }

    pub fn get_additional_records_mut(&mut self) -> &mut Vec<(String, Box<dyn RecordBase>)> {
        &mut self.additional_records
    }

    pub fn calculate_total_additional_records(&self) -> usize {
        self.additional_records.len()
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
