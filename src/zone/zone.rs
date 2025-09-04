use rlibdns::messages::inter::rr_types::RRTypes;
use rlibdns::records::inter::record_base::RecordBase;
use rlibdns::journal::txn::Txn;
use rlibdns::utils::ordered_map::OrderedMap;
use crate::zone::inter::zone_types::ZoneTypes;

#[derive(Debug, Clone)]
pub struct Zone {
    _type: ZoneTypes,
    records: OrderedMap<RRTypes, Vec<Box<dyn RecordBase>>>,
    children: OrderedMap<String, Self>,
    journal: OrderedMap<u32, Txn> //INDEX WILL BE A PROBLEM - CANNOT USE VEC...
}

impl Zone {

    pub fn new(_type: ZoneTypes) -> Self {
        Self {
            _type,
            records: OrderedMap::new(),
            children: OrderedMap::new(),
            journal: OrderedMap::new()
        }
    }

    pub fn set_type(&mut self, _type: ZoneTypes) {
        self._type = _type;
    }

    pub fn get_type(&self) -> ZoneTypes {
        self._type
    }

    pub fn is_authority(&self) -> bool {
        self._type.eq(&ZoneTypes::Master) || self._type.eq(&ZoneTypes::Slave)
    }

    pub fn has_sub_zone(&self, name: &str) -> bool {
        self.children.contains_key(name)
    }

    pub fn add_sub_zone(&mut self, name: &str, child: Self) {
        self.children.entry(name.to_string()).or_insert(child);
    }

    pub fn add_zone_to(&mut self, name: &str, zone: Self, default_type: ZoneTypes) {
        let labels: Vec<&str> = name.split('.').rev().collect();

        if labels.is_empty() {
            return;
        }

        let mut current = self;

        for label in &labels[..labels.len() - 1] {
            current = current.children.entry(label.to_string())
                .or_insert_with(|| Self::new(default_type.clone()));
        }

        current.children.insert(labels.last().unwrap().to_string(), zone);
    }

    pub fn get_sub_zone(&self, name: &str) -> Option<&Self> {
        self.children.get(name)
    }

    pub fn get_deepest_zone(&self, name: &str) -> Option<&Self> {
        let labels: Vec<&str> = name.trim_end_matches('.').split('.').rev().collect();

        let mut current = self;
        for label in labels {
            match current.children.get(label) {
                Some(child) => current = child,
                None => return None
            }
        }

        Some(current)
    }

    pub fn get_deepest_zone_mut(&mut self, name: &str) -> Option<&mut Self> {
        let labels: Vec<&str> = name.trim_end_matches('.').split('.').rev().collect();

        let mut current = self;
        for label in labels {
            match current.children.get_mut(label) {
                Some(child) => current = child,
                None => return None,
            }
        }

        Some(current)
    }

    pub fn get_deepest_zone_with_records(&self, name: &str, _type: &RRTypes) -> Option<(String, &Self)> {
        let labels: Vec<&str> = name.trim_end_matches('.').split('.').rev().collect();

        if self.records.contains_key(_type) {
            return Some((name.to_string(), self));
        }

        let mut current = self;
        let mut last_match: Option<(String, &Self)> = None;
        let mut current_labels = Vec::new();

        for label in &labels {
            current_labels.push(*label);

            match current.children.get(*label) {
                Some(child) => {
                    current = child;
                    if let Some(records) = current.get_records(_type) {
                        if !records.is_empty() {
                            last_match = Some((current_labels.iter().rev().cloned().collect::<Vec<_>>().join("."), current));
                        }
                    }
                }
                None => {}
            }
        }

        last_match
    }

    pub fn remove_sub_zone(&mut self, name: &str) {
        self.children.remove(name);
    }

    pub fn add_record(&mut self, record: Box<dyn RecordBase>) {
        self.records.entry(record.get_type()).or_insert(Vec::new()).push(record);
    }

    pub fn add_record_to(&mut self, name: &str, record: Box<dyn RecordBase>, default_type: ZoneTypes) {
        let labels: Vec<&str> = name.trim_end_matches('.').split('.').rev().collect();

        let mut current = self;

        for label in &labels[..labels.len().saturating_sub(1)] {
            current = current.children
                .entry(label.to_string())
                .or_insert_with(|| Self::new(default_type.clone()));
        }

        if let Some(leaf_label) = labels.last() {
            let leaf_zone = current.children
                .entry(leaf_label.to_string())
                .or_insert_with(|| Self::new(default_type.clone()));
            leaf_zone.add_record(record);
        }
    }

    pub fn get_records(&self, _type: &RRTypes) -> Option<&Vec<Box<dyn RecordBase>>> {
        self.records.get(_type)
    }

    pub fn get_all_records(&self) -> impl Iterator<Item = (&RRTypes, &Vec<Box<dyn RecordBase>>)> {
        self.records.iter()
    }

    pub fn get_all_records_recursive(&self) -> impl Iterator<Item = (String, Vec<&Box<dyn RecordBase>>)> + '_ {
        let mut res = OrderedMap::new();
        self.collect_records(String::new(), &mut res);
        res.into_iter()
    }

    fn collect_records<'a>(&'a self, fqdn: String, map: &mut OrderedMap<String, Vec<&'a Box<dyn RecordBase>>>) {
        let recs: Vec<&Box<dyn RecordBase>> = self
            .records
            .iter()
            .filter(|(ty, _)| **ty != RRTypes::Soa)
            .flat_map(|(_, v)| v.iter())
            .collect();

        if !recs.is_empty() {
            map.insert(fqdn.to_string(), recs);
        }

        for (label, child) in &self.children {
            let is_delegated = child.records.get(&RRTypes::Soa).map_or(false, |recs| !recs.is_empty());

            if is_delegated {
                continue;
            }

            child.collect_records(format!("{}.{}", label, fqdn), map);
        }
    }

    pub fn add_txn(&mut self, txn: Txn) {
        self.journal.insert(txn.get_serial_0(), txn);
    }

    pub fn get_txn_from(&self, serial_start: u32) -> impl Iterator<Item = (&u32, &Txn)> {
        self.journal.range(serial_start..)
    }
}
