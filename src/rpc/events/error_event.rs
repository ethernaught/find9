use rlibdns::messages::inter::response_codes::ResponseCodes;
use rlibdns::records::inter::record_base::RecordBase;
use rlibdns::utils::index_map::IndexMap;
use crate::rpc::events::inter::event::Event;

pub struct ErrorEvent {
    prevent_default: bool,
    code: ResponseCodes,
    name_servers: IndexMap<String, Vec<Box<dyn RecordBase>>>,
    additional_records: IndexMap<String, Vec<Box<dyn RecordBase>>>
}

impl ErrorEvent {

    pub fn new(code: ResponseCodes) -> Self {
        Self {
            prevent_default: false,
            code,
            name_servers: IndexMap::new(),
            additional_records: IndexMap::new()
        }
    }

    pub fn get_code(&self) -> ResponseCodes {
        self.code
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

    pub fn get_name_servers(&self) -> &IndexMap<String, Vec<Box<dyn RecordBase>>> {
        &self.name_servers
    }

    pub fn get_name_servers_mut(&mut self) -> &mut IndexMap<String, Vec<Box<dyn RecordBase>>> {
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

    pub fn get_additional_records(&self) -> &IndexMap<String, Vec<Box<dyn RecordBase>>> {
        &self.additional_records
    }

    pub fn get_additional_records_mut(&mut self) -> &mut IndexMap<String, Vec<Box<dyn RecordBase>>> {
        &mut self.additional_records
    }
}

impl Event for ErrorEvent {

    fn is_prevent_default(&self) -> bool {
        self.prevent_default
    }

    fn prevent_default(&mut self) {
        self.prevent_default = true;
    }
}
