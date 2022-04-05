use crate::*;

pub trait DataPointStorage {
    fn get(&self, key: Bytes32) -> Option<DataPoint>;
    fn store(&mut self, key: Bytes32, datapoint: DataPoint);
}

pub fn derive_beacon_id(airnode: Bytes, template_id: Bytes32) -> Bytes32 {
    let (encoded, _) = encode_packed(&[
        Token::Bytes(airnode),
        Token::FixedBytes(template_id.to_vec()),
    ]);
    keccak256(&encoded)
}

pub fn decode_fulfillment_data(data: Bytes) -> Result<Int, Error> {
    ensure!(data.len() == 32, Error::InvalidDataLength)?;

    let tokens = decode(&[ParamType::Int(0)], &data)?;
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
    let updated_beacon_value = decode_fulfillment_data(data)?;

    let beacon = s.get(beacon_id).ok_or(Error::BeaconDataNotFound)?;
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
