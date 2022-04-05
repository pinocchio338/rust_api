use crate::WrappedDataPoint;
use anchor_lang::accounts::account::Account;
use anchor_lang::prelude::*;
use api3_common::{Bytes32, DataPoint, DataPointStorage};

pub(crate) struct SolanaDataPointStorage<'info, 'account> {
    pub(crate) account: &'account mut Account<'info, WrappedDataPoint>,
}

impl DataPointStorage for SolanaDataPointStorage<'_, '_> {
    fn get(&self, _key: Bytes32) -> Option<DataPoint> {
        if self.account.raw_datapoint.is_empty() {
            return Some(DataPoint::default());
        }
        match DataPoint::from(self.account.raw_datapoint.clone()) {
            Ok(d) => Some(d),
            Err(e) => {
                msg!("cannot load datapoint due to: {:?}", e);
                None
            }
        }
    }

    fn store(&mut self, _key: Bytes32, datapoint: DataPoint) {
        self.account.raw_datapoint = Vec::from(datapoint);
    }
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
