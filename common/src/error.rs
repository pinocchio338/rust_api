#[derive(Debug)]
pub enum Error {
    CannotDeserializeDataPoint,
    InvalidData,
    InvalidDataLength,
    InvalidDataType,
    BeaconDataNotFound,
    FulfillmentOlderThanBeacon,
    InvalidName(String),
    EthAbiError(ethabi::Error),
    #[cfg(feature = "recovery")]
    Libsecp256k1Error(libsecp256k1::Error),
    ParameterLengthMismatch,
    LessThanTwoBeacons,
    InvalidTimestamp,
    InvalidSignature,
    UpdatedValueOutdated,
    AccessDenied,
    NameHashNotFound,
    RoleDescriptionEmpty,
    InvalidAddress,
    OnlyRenounceRolesForSelf,
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
            Error::EthAbiError(_) => 7,
            #[cfg(feature = "recovery")]
            Error::Libsecp256k1Error(_) => 8,
            Error::ParameterLengthMismatch => 9,
            Error::LessThanTwoBeacons => 10,
            Error::InvalidTimestamp => 11,
            Error::InvalidSignature => 12,
            Error::UpdatedValueOutdated => 13,
            Error::AccessDenied => 14,
            Error::NameHashNotFound => 15,
            Error::RoleDescriptionEmpty => 16,
            Error::InvalidAddress => 17,
            Error::OnlyRenounceRolesForSelf => 18,
        }
    }
}
