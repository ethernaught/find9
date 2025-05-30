use std::collections::HashMap;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::inter::record_base::RecordBase;
use crate::zone::inter::zone_types::ZoneTypes;

pub type RecordMap = HashMap<String, HashMap<RRTypes, Vec<Box<dyn RecordBase>>>>;

#[derive(Debug, Clone)]
pub struct Zone {
    _type: ZoneTypes,
    records: RecordMap
}

impl Zone {

    pub fn new(_type: ZoneTypes) -> Self {
        Self {
            _type,
            records: RecordMap::new()
        }
    }

    pub fn set_type(&mut self, _type: ZoneTypes) {
        self._type = _type;
    }

    pub fn get_type(&self) -> ZoneTypes {
        self._type
    }

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
}
