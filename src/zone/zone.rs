use std::collections::HashMap;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::inter::record_base::RecordBase;
use crate::zone::inter::zone_types::ZoneTypes;

pub type RecordMap = HashMap<String, HashMap<RRTypes, Vec<Box<dyn RecordBase>>>>;

#[derive(Debug, Clone)]
pub struct Zone {
    _type: ZoneTypes,
    records: HashMap<RRTypes, Vec<Box<dyn RecordBase>>>,
    children: HashMap<String, Vec<Self>>
}

impl Zone {

    pub fn new(_type: ZoneTypes) -> Self {
        Self {
            _type,
            records: HashMap::new(),
            children: HashMap::new()
        }
    }

    pub fn set_type(&mut self, _type: ZoneTypes) {
        self._type = _type;
    }

    pub fn get_type(&self) -> ZoneTypes {
        self._type
    }

    pub fn has_child(&self, name: &str) -> bool {
        self.children.contains_key(name)
    }

    pub fn add_child(&mut self, name: &str, child: Self) {
        self.children.entry(name.to_string()).or_insert(Vec::new()).push(child);
    }

    pub fn remove_child(&mut self, name: &str) {
        self.children.remove(name);
    }

    pub fn add_record(&mut self, record: Box<dyn RecordBase>) {
        self.records.entry(record.get_type()).or_insert(Vec::new()).push(record);
    }

    pub fn get_records(&self, _type: &RRTypes) -> Option<&Vec<Box<dyn RecordBase>>> {
        self.records.get(_type)
    }

    pub fn get_all_records(&self) -> &HashMap<RRTypes, Vec<Box<dyn RecordBase>>> {
        &self.records
    }
    /*
    pub fn add_record(&mut self, name: &str, record: Box<dyn RecordBase>) {
        self.records
            .entry(name.to_string())
            .or_insert_with(HashMap::new)
            .entry(record.get_type())
            .or_insert_with(Vec::new)
            .push(record);
    }

    pub fn get_records(&self, name: &str, _type: &RRTypes) -> Option<&Vec<Box<dyn RecordBase>>> {
        self.records.get(name)?.get(_type)
    }

    pub fn get_all_records(&self) -> &RecordMap {
        &self.records
    }
    */
}
