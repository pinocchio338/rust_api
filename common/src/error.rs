use std::fmt::Debug;
use thiserror::Error;

//TODO: copy the exact error messages from the original solidity code
#[derive(Error, Debug)]
pub enum Error {
    #[error("")]
    CannotDeserializeDataPoint,
    #[error("")]
    InvalidData,
    #[error("Data length not correct")]
    InvalidDataLength,
    #[error("Invalid data type")]
    InvalidDataType,
    #[error("Beacon data not found")]
    BeaconDataNotFound,
    #[error("Fulfillment older than Beacon")]
    FulfillmentOlderThanBeacon,
    #[error("Invalid name: {0}")]
    InvalidName(String),
    #[error("Parameter length mismatch")]
    ParameterLengthMismatch,
    #[error("Specified less than two Beacons")]
    LessThanTwoBeacons,
    #[error("Timestamp not valid")]
    InvalidTimestamp,
    #[error("Signature mismatch")]
    InvalidSignature,
    #[error("Updated value outdated")]
    UpdatedValueOutdated,
    #[error("Does not extend expiration")]
    DoesNotExtendExpiration,
    #[error("Access Denied")]
    AccessDenied,
    #[error("NameHash Not Found")]
    NameHashNotFound,
    #[error("Role description Empty")]
    RoleDescriptionEmpty,
    #[error("Service ID zero")]
    ServiceIdZero,
    #[error("User address zero")]
    UserAddressZero,
    #[error("Invalid Address")]
    InvalidAddress,
    #[error("Only Renounce roles for self")]
    OnlyRenounceRolesForSelf,
    #[error("Not authorized to perform this action")]
    NotAuthorized,
    #[error("Role admin not found")]
    RoleAdminNotFound,
    #[error("Contract already initialized")]
    AlreadyInitialized,
    #[error("Cannot set indefinite status")]
    CannotSetIndefiniteStatus,
    #[error("Template id cannot be zero")]
    TemplateIdZero,
    #[error("Airnode id cannot be zero")]
    AirnodeIdZero,
    #[error("Setter can set indefinite status")]
    SetterCanSetIndefiniteStatus
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
            Error::ParameterLengthMismatch => 9,
            Error::LessThanTwoBeacons => 10,
            Error::InvalidTimestamp => 11,
            Error::InvalidSignature => 12,
            Error::UpdatedValueOutdated => 13,
            Error::AccessDenied => 14,
            Error::NameHashNotFound => 15,
            Error::RoleDescriptionEmpty => 16,
            Error::DoesNotExtendExpiration => 17,
            Error::ServiceIdZero => 18,
            Error::UserAddressZero => 19,
            Error::InvalidAddress => 20,
            Error::OnlyRenounceRolesForSelf => 21,
            Error::NotAuthorized => 22,
            Error::RoleAdminNotFound => 23,
            Error::AlreadyInitialized => 24,
            Error::CannotSetIndefiniteStatus => 25,
            Error::TemplateIdZero => 26,
            Error::AirnodeIdZero => 27,
            Error::SetterCanSetIndefiniteStatus => 28,
        }
    }
}
