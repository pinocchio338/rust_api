use crate::abi::{decode, encode, encode_packed, keccak256, Int, ParamType, Token, Uint, U256};
use crate::access::AccessControlRegistry;
use crate::whitelist::Whitelist;
use crate::{ensure, keccak_packed, median, Bytes, Bytes32, DataPoint, Error, StaticRole, Zero};

const ONE_HOUR_IN_SECONDS: u32 = 3600;
const FIFTEEN_MINUTES_IN_SECONDS: u32 = 900;

/// Generic storage trait. Used for the common processing logic so that each chain could
/// have their own implementation.
pub trait Storage<T> {
    fn get(&self, key: &Bytes32) -> Option<T>;
    fn store(&mut self, key: Bytes32, t: T);
}

/// Public trait that handles signature verification across different chains
pub trait SignatureManger {
    /// Verifies the signature against the message and public key
    /// Returns if the signature is valid
    ///
    /// # Arguments
    ///
    /// * `key` - The public key of the signer
    /// * `message` - The message to verify
    /// * `signature` - The signature to verify
    fn verify(key: &[u8], message: &[u8], signature: &[u8]) -> bool;
}

/// Public trait that handles timestamp fetching across different chains
pub trait TimestampChecker {
    fn current_timestamp(&self) -> u32;

    /// Returns if the timestamp used in the signature is valid
    /// Returns `false` if the timestamp is not at most 1 hour old to
    /// prevent replays. Returns `false` if the timestamp is not from the past,
    /// with some leeway to accomodate for some benign time drift. These values
    /// are appropriate in most cases, but you can adjust them if you are aware
    /// of the implications.
    ///
    /// # Arguments
    ///
    /// * `timestamp` Timestamp used in the signature
    fn is_valid(&self, timestamp: u32) -> bool {
        let c = self.current_timestamp();
        timestamp
            .checked_add(ONE_HOUR_IN_SECONDS)
            .expect("Invalid timestamp")
            > c
            && timestamp < c + FIFTEEN_MINUTES_IN_SECONDS
    }
}

/// Reads the data point with ID
/// Returns tuple containing (DataPoint.value, DataPoint.timestamp).
///
/// # Arguments
///
/// * `datapoint_id` Data point ID
/// * `msg_sender` Address of who sent the transaction
/// * `datapoint_storage` Data point storage that links `datapoint_id` to `Datapoint`
/// * `access` The access control registry used
/// * `whitelist` The whitelist implementation used
pub fn read_with_data_point_id<
    D: Storage<DataPoint>,
    A: AccessControlRegistry,
    W: Whitelist<Address = A::Address>,
>(
    datapoint_id: &Bytes32,
    msg_sender: &A::Address,
    datapoint_storage: &D,
    access: &A,
    whitelist: &W,
) -> Result<(Int, u32), Error> {
    ensure!(
        reader_can_read_data_point(datapoint_id, msg_sender, access, whitelist),
        Error::AccessDenied
    )?;
    let data_point = datapoint_storage
        .get(datapoint_id)
        .ok_or(Error::BeaconDataNotFound)?;
    Ok((data_point.value, data_point.timestamp))
}

/// Reads the data point with name
/// The read data point may belong to a Beacon or dAPI. The reader
/// must be whitelisted for the hash of the data point name.
/// Returns tuple containing (DataPoint.value, DataPoint.timestamp).
///
/// # Arguments
///
/// * `name` Data point name
/// * `msg_sender` Address of who sent the transaction
/// * `datapoint_storage` Data point storage that links `datapoint_id` to `Datapoint`
/// * `name_storage` Name to Datapoint Id storage used
/// * `access` The access control registry used
/// * `whitelist` The whitelist implementation used
pub fn read_with_name<
    D: Storage<DataPoint>,
    H: Storage<Bytes32>,
    A: AccessControlRegistry,
    W: Whitelist<Address = A::Address>,
>(
    name: Bytes32,
    msg_sender: &A::Address,
    datapoint_storage: &D,
    name_storage: &H,
    access: &A,
    whitelist: &W,
) -> Result<(Int, u32), Error> {
    let name_hash = keccak_packed(&[Token::FixedBytes(name.to_vec())]);
    ensure!(
        reader_can_read_data_point(&name_hash, msg_sender, access, whitelist),
        Error::AccessDenied
    )?;
    let key = name_storage
        .get(&name_hash)
        .ok_or(Error::NameHashNotFound)?;
    let data_point = datapoint_storage
        .get(&key)
        .ok_or(Error::BeaconDataNotFound)?;
    Ok((data_point.value, data_point.timestamp))
}

/// Returns if a reader can read the data point
///
/// # Arguments
///
/// * `data_point_id` Data point ID
/// * `reader` The reader that is trying to read the datapoint
/// * `access` The access control registry used
/// * `whitelist` The whitelist implementation used
pub fn reader_can_read_data_point<A: AccessControlRegistry, W: Whitelist<Address = A::Address>>(
    data_point_id: &Bytes32,
    reader: &A::Address,
    access: &A,
    whitelist: &W,
) -> bool {
    let role = access.find_static_role(StaticRole::UnlimitedReaderRole);
    reader.is_zero()
        || whitelist.user_is_whitelisted(data_point_id, reader)
        || access.has_role(&role, reader)
}

/// Updates the dAPI that is specified by the beacon IDs.
/// Returns the dAPI ID.
///
/// # Arguments
///
/// * `beacon_ids` is the list of beacon ids to perform aggregation
pub fn update_dapi_with_beacons<D: Storage<DataPoint>>(
    d: &mut D,
    beacon_ids: &[Bytes32],
) -> Result<Bytes32, Error> {
    let beacon_count = beacon_ids.len();
    ensure!(beacon_count > 1, Error::LessThanTwoBeacons)?;

    let mut values = Vec::with_capacity(beacon_count);
    let mut accumulated_timestamp = U256::from(0);

    for beacon_id in beacon_ids {
        let data_point = d.get(beacon_id).ok_or(Error::BeaconDataNotFound)?;
        values.push(data_point.value);
        accumulated_timestamp += U256::from(data_point.timestamp);
    }

    let dapi_id = derive_dapi_id(beacon_ids);
    let dapi_datapoint = d.get(&dapi_id).ok_or(Error::BeaconDataNotFound)?;

    let updated_timestamp = (accumulated_timestamp / beacon_count).as_u32();
    ensure!(
        updated_timestamp >= dapi_datapoint.timestamp,
        Error::UpdatedValueOutdated
    )?;
    let updated_value = median(&values);
    let datapoint = DataPoint::new(updated_value, updated_timestamp);

    d.store(dapi_id, datapoint);
    Ok(dapi_id)
}

/// Updates a dAPI using data signed by the respective Airnodes
/// without requiring a request or subscription. The beacons for which the
/// signature is omitted will be read from the storage.
/// Returns the dAPI ID.
///
/// # Arguments
///
/// * `datapoint_storage` The datapoint storage trait implementation to use
/// * `timestamp_checker` The timestamp checker/validator to use
/// * `airnodes` Airnode addresses
/// * `template_ids` Template IDs
/// * `timestamps` Timestamps used in the signatures
/// * `data` Response data (an `int256` encoded in contract ABI per Beacon)
/// * `signatures` Template ID, a timestamp and the response data signed by the respective Airnode address per Beacon
#[allow(clippy::too_many_arguments)]
pub fn update_dapi_with_signed_data<
    D: Storage<DataPoint>,
    S: SignatureManger,
    T: TimestampChecker,
>(
    datapoint_storage: &mut D,
    timestamp_checker: &T,
    airnodes: Vec<Bytes>,
    template_ids: Vec<[u8; 32]>,
    timestamps: Vec<[u8; 32]>,
    data: Vec<Bytes>,
    signatures: Vec<Bytes>,
) -> Result<Bytes32, Error> {
    let beacon_count = template_ids.len();

    ensure!(
        beacon_count == template_ids.len()
            && beacon_count == timestamps.len()
            && beacon_count == data.len()
            && beacon_count == signatures.len(),
        Error::ParameterLengthMismatch
    )?;

    ensure!(beacon_count > 1, Error::LessThanTwoBeacons)?;

    let mut beacon_ids = Vec::with_capacity(beacon_count);
    let mut values = Vec::with_capacity(beacon_count);
    let mut accumulated_timestamp = U256::from(0);

    for ind in 0..beacon_count {
        if !signatures[ind].is_empty() {
            let timestamp = U256::from_big_endian(&timestamps[ind]);
            let timestamp_u32 = timestamp.as_u32();
            ensure!(
                timestamp_checker.is_valid(timestamp_u32),
                Error::InvalidTimestamp
            )?;

            let message = keccak_packed(&[
                Token::FixedBytes(template_ids[ind].clone().to_vec()),
                Token::Uint(timestamp),
                Token::Bytes(data[ind].clone()),
            ]);
            ensure!(
                S::verify(&airnodes[ind], &message, &signatures[ind]),
                Error::InvalidSignature
            )?;

            values.push(decode_fulfillment_data(&data[ind])?);

            // Timestamp validity is already checked, which means it will
            // be small enough to be typecast into `uint32`
            accumulated_timestamp += timestamp;
            beacon_ids.push(derive_beacon_id(airnodes[ind].clone(), template_ids[ind]));
        } else {
            let beacon_id = derive_beacon_id(airnodes[ind].clone(), template_ids[ind]);
            let data_point = datapoint_storage
                .get(&beacon_id)
                .ok_or(Error::BeaconDataNotFound)?;
            values.push(data_point.value);
            accumulated_timestamp += U256::from(data_point.timestamp);
            beacon_ids.push(beacon_id);
        }
    }
    let dapi_id = derive_dapi_id(&beacon_ids);
    let updated_timestamp = (accumulated_timestamp / beacon_count).as_u32();
    let dapi_datapoint = datapoint_storage
        .get(&dapi_id)
        .ok_or(Error::BeaconDataNotFound)?;
    ensure!(
        updated_timestamp >= dapi_datapoint.timestamp,
        Error::UpdatedValueOutdated
    )?;
    let updated_value = median(&values);
    let datapoint = DataPoint::new(updated_value, updated_timestamp);
    datapoint_storage.store(dapi_id, datapoint);
    Ok(dapi_id)
}

/// Sets the data point ID the name points to
/// While a data point ID refers to a specific Beacon or dAPI, names
/// provide a more abstract interface for convenience. This means a name
/// that was pointing at a Beacon can be pointed to a dAPI, then another
/// dAPI, etc.
///
/// # Arguments
///
/// * `name` Human-readable name
/// * `datapoint_id` Data point ID the name will point to
/// * `msg_sender` Address of who sent the transaction
/// * `access` Access control implementation to use
/// * `storage` Storage implementation to use for linking name and datapoint_id
pub fn set_name<D: Storage<Bytes32>, A: AccessControlRegistry>(
    name: Bytes32,
    datapoint_id: Bytes32,
    msg_sender: &A::Address,
    access: &A,
    storage: &mut D,
) -> Result<(), Error> {
    ensure!(name != Bytes32::default(), Error::InvalidData)?;
    ensure!(datapoint_id != Bytes32::default(), Error::InvalidData)?;
    let role = access.find_static_role(StaticRole::NameSetterRole);
    ensure!(access.has_role(&role, msg_sender), Error::AccessDenied)?;

    storage.store(
        keccak_packed(&[Token::FixedBytes(name.to_vec())]),
        datapoint_id,
    );

    Ok(())
}

/// Derives the beacon id based on the `airnode` and `templated_id`
/// Returns the beacon id
///
/// # Arguments
///
/// * `airnode` Airnode address
/// * `template_id` Template ID
pub fn derive_beacon_id(airnode: Bytes, template_id: Bytes32) -> Bytes32 {
    ensure!(not_zero(&airnode), Error::AirnodeIdZero).unwrap();
    ensure!(not_zero(&template_id), Error::TemplateIdZero).unwrap();
    let (encoded, _) = encode_packed(&[
        Token::Bytes(airnode),
        Token::FixedBytes(template_id.to_vec()),
    ]);
    keccak256(&encoded)
}

/// Derives the dAPI ID from the beacon IDs
/// Notice that `encode()` is used over `encode_packed()`
/// Returns the derived dapi id
///
/// # Arguments
///
/// * `beacon_ids` Beacon IDs
pub fn derive_dapi_id(beacon_ids: &[Bytes32]) -> Bytes32 {
    let tokens: Vec<Token> = beacon_ids
        .iter()
        .map(|b| Token::FixedBytes(b.to_vec()))
        .collect();
    let encoded = encode(&tokens);
    keccak256(&encoded)
}

/// Decode the encoded data to the respective data types.
/// Returns the `Result` of decoding fulfillment data.
///
/// # Arguments
///
/// * `data` Fulfillment data (an `int256` encoded in contract ABI)
pub fn decode_fulfillment_data(data: &Bytes) -> Result<Int, Error> {
    ensure!(data.len() == 32, Error::InvalidDataLength)?;

    let tokens = decode(&[ParamType::Int(0)], data)?;
    ensure!(tokens.len() == 1, Error::InvalidDataLength)?;

    if let Token::Int(i) = tokens[0] {
        Ok(i)
    } else {
        Err(Error::InvalidDataType)
    }
}

/// Called privately to process the Beacon update.
/// Returns the updated Beacon value.
///
/// # Arguments
///
/// * `storage` The storage between `beacon_id` to `Datapoint`
/// * `beacon_id` The Beacon ID
/// * `timestamp` Timestamp used in the signature
/// * `data` Fulfillment data (an `int256` encoded in contract ABI)
pub fn process_beacon_update<D: Storage<DataPoint>>(
    storage: &mut D,
    beacon_id: Bytes32,
    timestamp: Uint,
    data: Bytes,
) -> Result<(), Error> {
    let updated_beacon_value = decode_fulfillment_data(&data)?;

    let beacon = storage.get(&beacon_id).ok_or(Error::BeaconDataNotFound)?;
    ensure!(
        timestamp.as_u32() > beacon.timestamp,
        Error::FulfillmentOlderThanBeacon
    )?;

    // Timestamp validity is already checked by `onlyValidTimestamp`, which
    // means it will be small enough to be typecast into `uint32`

    let datapoint = DataPoint::new(updated_beacon_value, timestamp.as_u32());
    storage.store(beacon_id, datapoint);

    Ok(())
}

fn not_zero(bytes: &[u8]) -> bool {
    let mut count = 0;
    for i in bytes {
        if *i == 0u8 {
            count += 1;
        }
    }
    count != bytes.len()
}

#[cfg(test)]
mod tests {
    use crate::beacon::not_zero;
    use crate::derive_beacon_id;

    #[test]
    fn not_zero_works() {
        assert!(!not_zero(&[0; 12]));
        let mut v = [0; 12];
        v[2] = 1;
        assert!(not_zero(&v));
    }

    #[test]
    fn encode_packed_works() {
        let raw_template_id =
            hex::decode("0000000000000000000000000000000000000000000000000000000000000001")
                .unwrap();
        let airnode =
            hex::decode("1d73899cc9fc3ad06a2c7f5bf26c8a4a76b42de905cb9b6ae96390355441a0ca")
                .unwrap();
        let mut template_id = [0; 32];
        template_id.copy_from_slice(&raw_template_id);
        let beacon_id = derive_beacon_id(airnode, template_id);
        assert_eq!(
            hex::encode(beacon_id),
            "ad1b5c75a8b8e0d7dbc56c1e28aee9fabe285ad8fb61a256ddabd4523bfb284a"
        );
    }
}
