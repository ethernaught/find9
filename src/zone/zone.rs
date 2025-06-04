use std::collections::HashMap;
use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::inter::record_base::RecordBase;
use crate::zone::inter::zone_types::ZoneTypes;

pub type RecordMap = HashMap<String, HashMap<RRTypes, Vec<Box<dyn RecordBase>>>>;

#[derive(Debug, Clone)]
pub struct Zone {
    _type: ZoneTypes,
    records: HashMap<RRTypes, Vec<Box<dyn RecordBase>>>,
    children: HashMap<String, Self>
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

    pub fn has_sub_zone(&self, name: &str) -> bool {
        self.children.contains_key(name)
    }

    pub fn add_sub_zone(&mut self, name: &str, child: Self) {
        self.children.entry(name.to_string()).or_insert(child);
    }

    pub fn get_sub_zone(&self, name: &str) -> Option<&Self> {
        self.children.get(name)
    }

    pub fn remove_sub_zone(&mut self, name: &str) {
        self.children.remove(name);
    }





    pub fn get_deepest_zone(&self, domain: &str) -> Option<&Zone> {
        let labels: Vec<&str> = domain.trim_end_matches('.').split('.').rev().collect();

        let mut current = self;
        for label in labels {
            match current.children.get(label) {
                Some(child) => current = child,
                None => return None
            }
        }

        Some(current)
    }

    pub fn get_deepest_zone_mut(&mut self, domain: &str) -> Option<&mut Zone> {
        let labels: Vec<&str> = domain.trim_end_matches('.').split('.').rev().collect();

        let mut current = self;
        for label in labels {
            match current.children.get_mut(label) {
                Some(child) => current = child,
                None => return None,
            }
        }

        Some(current)
    }

    //LOOP OBTAIN RECORDS....


    pub fn add_zone_to(&mut self, domain: &str, zone: Zone, default_type: ZoneTypes) {
        let labels: Vec<&str> = domain.split('.').rev().collect();

        if labels.is_empty() {
            return;
        }

        let mut current = self;

        for label in &labels[..labels.len() - 1] {
            current = current.children.entry(label.to_string())
                .or_insert_with(|| Zone::new(default_type.clone()));
        }

        current.children.insert(labels.last().unwrap().to_string(), zone);
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
}
