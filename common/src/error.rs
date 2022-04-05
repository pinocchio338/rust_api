#[derive(Debug)]
pub enum Error {
    CannotDeserializeDataPoint,
    InvalidData,
    InvalidDataLength,
    InvalidDataType,
    BeaconDataNotFound,
    FulfillmentOlderThanBeacon,
    InvalidName(String),

    #[cfg(feature = "recovery")]
    Libsecp256k1Error(libsecp256k1::Error),

    #[cfg(feature = "ethabi")]
    EthAbiError(ethabi::Error),
}

#[cfg(feature = "recovery")]
impl From<libsecp256k1::Error> for Error {
    fn from(e: libsecp256k1::Error) -> Self {
        Error::Libsecp256k1Error(e)
    }
}

impl From<Error> for u32 {
    fn from(e: Error) -> Self {
        match e {
            Error::CannotDeserializeDataPoint => 0,
            Error::InvalidData => 1,
            Error::InvalidDataLength => 2,
            Error::InvalidDataType => 3,
            Error::BeaconDataNotFound => 4,
            Error::FulfillmentOlderThanBeacon => 5,
            Error::InvalidName(_) => 6,
            #[cfg(feature = "recovery")]
            Error::Libsecp256k1Error(_) => 7,
            #[cfg(feature = "ethabi")]
            Error::EthAbiError(_) => 8,
        }
    }
}
