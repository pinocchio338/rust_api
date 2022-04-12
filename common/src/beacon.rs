use crate::*;

const ONE_HOUR_SEC: u32 = 3600000;
const FIFTEEN_MINUTES_SEC: u32 = 900000;

/// Generic storage trait. Used for the common processing logic so that each chain could
/// have their own implementation.
pub trait DataPointStorage {
    fn get(&self, key: &Bytes32) -> Option<DataPoint>;
    fn store(&mut self, key: Bytes32, datapoint: DataPoint);
}

/// Public trait that handles signature verification across different chains
pub trait SignatureManger {
    /// Checks if the signature passed from client is actually empty
    /// * `index` - The index of the signature to check if it is empty
    fn is_empty(&self, index: usize) -> bool;
    /// Verifies the signature against the message and public key
    /// * `key` - The public key of the signer
    /// * `message` - The message to verify
    /// * `signature` - The signature to verify
    fn verify(&self, key: &[u8], message: &[u8], signature: &[u8]) -> bool;
}

/// Public trait that handles timestamp fetching across different chains
pub trait TimestampChecker {
    fn current_timestamp(&self) -> u32;
    fn is_valid(&self, timestamp: u32) -> bool {
        let c = self.current_timestamp();
        timestamp
            .checked_add(ONE_HOUR_SEC)
            .expect("Invalid timestamp")
            > c
            && timestamp < c + FIFTEEN_MINUTES_SEC
    }
}

/// Updates the dAPI that is specified by the beacon IDs
/// `beacon_ids` is the list of beacon ids to perform aggregation
pub fn update_dapi_with_beacons<D: DataPointStorage>(
    d: &mut D,
    beacon_ids: &[Bytes32],
) -> Result<(), Error> {
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
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub fn update_dapi_with_signed_data<
    D: DataPointStorage,
    S: SignatureManger,
    T: TimestampChecker,
>(
    d: &mut D,
    s: &S,
    t: &T,
    airnodes: Vec<Bytes>,
    template_ids: Vec<[u8; 32]>,
    timestamps: Vec<[u8; 32]>,
    data: Vec<Bytes>,
    signatures: Vec<Bytes>,
) -> Result<(), Error> {
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
        if !s.is_empty(ind) {
            let timestamp = U256::from(&timestamps[ind]);
            let timestamp_u32 = timestamp.as_u32();
            ensure!(t.is_valid(timestamp_u32), Error::InvalidTimestamp)?;

            let (encoded, _) = encode_packed(&[
                Token::FixedBytes(template_ids[ind].clone().to_vec()),
                Token::Uint(timestamp),
                Token::Bytes(data[ind].clone()),
            ]);
            let message = to_eth_signed_message_hash(&keccak256(&encoded));
            ensure!(
                s.verify(&airnodes[ind], &message, &signatures[ind]),
                Error::InvalidSignature
            )?;

            values.push(decode_fulfillment_data(&data[ind])?);

            // Timestamp validity is already checked, which means it will
            // be small enough to be typecast into `uint32`
            accumulated_timestamp += timestamp;
            beacon_ids.push(derive_beacon_id(
                airnodes[ind].clone().to_vec(),
                template_ids[ind],
            ));
        } else {
            let beacon_id = derive_beacon_id(airnodes[ind].clone(), template_ids[ind]);
            let data_point = d.get(&beacon_id).ok_or(Error::BeaconDataNotFound)?;
            values.push(data_point.value);
            accumulated_timestamp += U256::from(data_point.timestamp);
            beacon_ids.push(beacon_id);
        }
    }
    let dapi_id = derive_dapi_id(&beacon_ids);
    let updated_timestamp = (accumulated_timestamp / beacon_count).as_u32();
    let dapi_datapoint = d.get(&dapi_id).ok_or(Error::BeaconDataNotFound)?;
    ensure!(
        updated_timestamp >= dapi_datapoint.timestamp,
        Error::UpdatedValueOutdated
    )?;
    let updated_value = median(&values);
    let datapoint = DataPoint::new(updated_value, updated_timestamp);
    d.store(dapi_id, datapoint);
    Ok(())
}

pub fn derive_beacon_id(airnode: Bytes, template_id: Bytes32) -> Bytes32 {
    let (encoded, _) = encode_packed(&[
        Token::Bytes(airnode),
        Token::FixedBytes(template_id.to_vec()),
    ]);
    keccak256(&encoded)
}

/// @notice Derives the dAPI ID from the beacon IDs
/// @dev Notice that `abi.encode()` is used over `abi.encodePacked()`
/// @param beaconIds Beacon IDs
/// @return dapiId dAPI ID
fn derive_dapi_id(beacon_ids: &[Bytes32]) -> Bytes32 {
    let tokens: Vec<Token> = beacon_ids
        .iter()
        .map(|b| Token::FixedBytes(b.to_vec()))
        .collect();
    let encoded = encode(&tokens);
    keccak256(&encoded)
}

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

pub fn process_beacon_update<D: DataPointStorage>(
    s: &mut D,
    beacon_id: Bytes32,
    timestamp: Uint,
    data: Bytes,
) -> Result<(), Error> {
    let updated_beacon_value = decode_fulfillment_data(&data)?;

    let beacon = s.get(&beacon_id).ok_or(Error::BeaconDataNotFound)?;
    ensure!(
        timestamp.as_u32() > beacon.timestamp,
        Error::FulfillmentOlderThanBeacon
    )?;

    // Timestamp validity is already checked by `onlyValidTimestamp`, which
    // means it will be small enough to be typecast into `uint32`

    let datapoint = DataPoint::new(updated_beacon_value, timestamp.as_u32());
    s.store(beacon_id, datapoint);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::derive_beacon_id;

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
