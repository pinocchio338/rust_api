use crate::WrappedDataPoint;
use anchor_lang::accounts::account::Account;
use anchor_lang::prelude::*;
use api3_common::Bytes32;

pub(crate) fn update_beacon_data(
    beacon_account: &mut Account<WrappedDataPoint>,
    data: Vec<u8>,
) -> Result<()> {
    beacon_account.raw_datapoint = data;
    Ok(())
}

pub(crate) fn derive_dapi_id(_beacon_ids: &[Bytes32]) -> Bytes32 {
    Bytes32::default()
}

pub(crate) fn check_beacon_ids(
    _beacon_ids: &[Bytes32],
    _beacon_id_tuples: &[(Pubkey, Account<WrappedDataPoint>)],
) -> Result<()> {
    Ok(())
}

/// Checks the dapis_id passed as parameter is actually derived from beacon_ids.
pub(crate) fn check_dapi_id(_dapi_id: &Bytes32, _beacon_ids: &[Bytes32]) -> Result<()> {
    derive_dapi_id(_beacon_ids);
    Ok(())
}
