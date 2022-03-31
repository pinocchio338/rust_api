#[derive(Debug)]
pub enum Error {
    CannotDeserializeDataPoint,
    InvalidData,
    InvalidName(String),
}
