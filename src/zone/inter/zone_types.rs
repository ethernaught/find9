#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ZoneTypes {
    Master,
    Slave,
    Stub,
    Forward,
    Hint,
}
