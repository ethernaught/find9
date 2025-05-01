use std::io;
use rlibdns::messages::inter::dns_classes::DnsClasses;

pub trait DnsClassesExt {

    fn from_str(value: &str) -> io::Result<DnsClasses>;

    fn short_value(&self) -> String;
}

impl DnsClassesExt for DnsClasses {

    fn from_str(value: &str) -> io::Result<Self> {
        for (c) in [Self::In] {
            if c.short_value() == value {
                return Ok(c);
            }
        }

        Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Couldn't find for value: {}", value)))
    }

    fn short_value(&self) -> String {
        match self {
            DnsClasses::In => "in",
            DnsClasses::Cs => "cs",
            DnsClasses::Ch => "ch",
            DnsClasses::Hs => "hs"
        }.to_string()
    }
}
