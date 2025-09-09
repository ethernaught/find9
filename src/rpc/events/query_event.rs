use rlibdns::messages::dns_query::DnsQuery;
use rlibdns::records::inter::record_base::RecordBase;
use crate::rpc::events::inter::event::Event;

pub struct QueryEvent {
    prevent_default: bool,
    query: DnsQuery,
    authoritative: bool,
    pub(crate) request_records: [Vec<(String, Box<dyn RecordBase>)>; 3],
    pub(crate) response_records: [Vec<(String, Box<dyn RecordBase>)>; 3],
    //answers: Vec<(String, Box<dyn RecordBase>)>, //WE HAVE TO SWITCH FROM ORDERED_MAP TO VEC FOR RFC 5936
    //authority_records: Vec<(String, Box<dyn RecordBase>)>,
    //additional_records: Vec<(String, Box<dyn RecordBase>)>,
    received_time: u128
}

impl QueryEvent {

    pub fn new(query: DnsQuery) -> Self {
        Self {
            prevent_default: false,
            query,
            authoritative: false,
            request_records: Default::default(),
            response_records: Default::default(),
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
        !self.response_records[0].is_empty()
    }

    pub fn add_answer(&mut self, query: &str, record: Box<dyn RecordBase>) {
        self.response_records[0].push((query.to_string(), record));
    }

    pub fn get_answers(&self) -> impl Iterator<Item = (&String, &Box<dyn RecordBase>)> {
        self.response_records[0].iter().map(|(query, record)| (query, record))
    }

    pub fn get_answers_mut(&mut self) -> impl Iterator<Item = (&mut String, &mut Box<dyn RecordBase>)> {
        self.response_records[0].iter_mut().map(|(query, record)| (query, record))
    }

    pub fn total_answers(&self) -> usize {
        self.response_records[0].len()
    }

    pub fn has_authority_records(&self) -> bool {
        !self.response_records[1].is_empty()
    }

    pub fn add_authority_record(&mut self, query: &str, record: Box<dyn RecordBase>) {
        self.response_records[1].push((query.to_string(), record));
    }

    pub fn get_authority_records(&self) -> impl Iterator<Item = (&String, &Box<dyn RecordBase>)> {
        self.response_records[1].iter().map(|(query, record)| (query, record))
    }

    pub fn get_authority_records_mut(&mut self) -> impl Iterator<Item = (&mut String, &mut Box<dyn RecordBase>)> {
        self.response_records[1].iter_mut().map(|(query, record)| (query, record))
    }

    pub fn total_authority_records(&self) -> usize {
        self.response_records[1].len()
    }

    pub fn has_additional_records(&self) -> bool {
        !self.response_records[2].is_empty()
    }

    pub fn add_additional_record(&mut self, query: &str, record: Box<dyn RecordBase>) {
        self.response_records[2].push((query.to_string(), record));
    }

    pub fn get_additional_records(&self) -> impl Iterator<Item = (&String, &Box<dyn RecordBase>)> {
        self.response_records[2].iter().map(|(query, record)| (query, record))
    }

    pub fn get_additional_records_mut(&mut self) -> impl Iterator<Item = (&mut String, &mut Box<dyn RecordBase>)> {
        self.response_records[2].iter_mut().map(|(query, record)| (query, record))
    }

    pub fn total_additional_records(&self) -> usize {
        self.response_records[2].len()
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
