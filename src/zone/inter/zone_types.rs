use std::fmt;
use std::fmt::Formatter;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ZoneTypes {
    Master,
    Slave,
    Stub,
    Forward,
    Hint
}

impl fmt::Display for ZoneTypes {

    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Master => "MASTER",
            Self::Slave => "SLAVE",
            Self::Stub => "STUB",
            Self::Forward => "FORWARD",
            Self::Hint => "HINT"
        })
    }
}
