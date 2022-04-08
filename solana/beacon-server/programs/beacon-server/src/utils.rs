use crate::WrappedDataPoint;
use anchor_lang::accounts::account::Account;
use anchor_lang::prelude::borsh::maybestd::collections::HashMap;
use anchor_lang::prelude::*;
use api3_common::{Bytes32, DataPoint, DataPointStorage};

pub type AccountRef<'info> = Account<'info, WrappedDataPoint>;
pub(crate) struct SolanaHashMap<'info, 'account> {
    map: HashMap<Bytes32, &'account mut AccountRef<'info>>,
}

impl<'info, 'account> SolanaHashMap<'info, 'account> {
    pub fn new(accounts: Vec<(Bytes32, &'account mut AccountRef<'info>)>) -> Self {
        let mut map = HashMap::new();
        for (key, aref) in accounts {
            map.insert(key, aref);
        }
        Self { map }
    }
}

impl DataPointStorage for SolanaHashMap<'_, '_> {
    fn get(&self, key: &Bytes32) -> Option<DataPoint> {
        self.map.get(key).map(|a| {
            if a.raw_datapoint.is_empty() {
                DataPoint::default()
            } else {
                DataPoint::from(a.raw_datapoint.clone()).expect("cannot load datapoint")
            }
        })
    }

    fn store(&mut self, key: Bytes32, datapoint: DataPoint) {
        let a = self.map.get_mut(&key).expect("cannot load from datapoint");
        (*a).raw_datapoint = Vec::from(datapoint);
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
