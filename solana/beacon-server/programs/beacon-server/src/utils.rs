use crate::{
    WrappedDataPoint, WrappedDataPointId, ERROR_DATA_LENGTH_NOT_MATCH, ERROR_INVALID_BEACON_ID_KEY,
    ERROR_INVALID_DERIVED_DAPI_ID_KEY, ERROR_INVALID_NAME_HASH, ERROR_INVALID_SYSTEM_PROGRAM_ID,
};
use anchor_lang::accounts::account::Account;
use anchor_lang::prelude::borsh::maybestd::collections::HashMap;
use anchor_lang::prelude::*;
use api3_common::abi::Token;
use api3_common::{
    ensure, keccak_packed, Bytes32, DataPoint, SignatureManger, Storage, TimestampChecker,
};

const DATAPOINT_SEED: &str = "datapoint";
// const HASH_NAME_SEED: &str = "hashed-name";

pub type NameHashAccountRef<'info> = Account<'info, WrappedDataPointId>;
pub(crate) struct NameHashHashMap<'info, 'account> {
    write: HashMap<Bytes32, &'account mut NameHashAccountRef<'info>>,
}

impl<'info, 'account> NameHashHashMap<'info, 'account> {
    pub fn new(accounts: Vec<(Bytes32, &'account mut NameHashAccountRef<'info>)>) -> Self {
        let mut write = HashMap::new();
        for (key, aref) in accounts {
            write.insert(key, aref);
        }
        Self { write }
    }
}

impl<'info, 'account> Storage<Bytes32> for NameHashHashMap<'info, 'account> {
    fn get(&self, k: &Bytes32) -> Option<Bytes32> {
        self.write.get(k).map(|a| a.datapoint_id)
    }

    fn store(&mut self, k: Bytes32, datapoint_id: Bytes32) {
        let a = self.write.get_mut(&k).expect("cannot load from datapoint");
        (*a).datapoint_id = datapoint_id;
        // panic!("datapoint id: {:?}", a.datapoint_id);
    }
}

pub type DatapointAccountRef<'info> = Account<'info, WrappedDataPoint>;
pub(crate) struct DatapointHashMap<'info, 'account> {
    write: HashMap<Bytes32, &'account mut DatapointAccountRef<'info>>,
    read: HashMap<Bytes32, DataPoint>,
}

impl<'info, 'account> DatapointHashMap<'info, 'account> {
    pub fn new(
        accounts: Vec<(Bytes32, &'account mut DatapointAccountRef<'info>)>,
        read: HashMap<Bytes32, DataPoint>,
    ) -> Self {
        let mut write = HashMap::new();
        for (key, aref) in accounts {
            write.insert(key, aref);
        }
        Self { write, read }
    }
}

impl<'info, 'account> Storage<DataPoint> for DatapointHashMap<'info, 'account> {
    fn get(&self, k: &Bytes32) -> Option<DataPoint> {
        match self.read.get(k) {
            Some(d) => Some(d.clone()),
            None => self.write.get(k).map(|a| {
                if a.raw_datapoint.is_empty() {
                    DataPoint::default()
                } else {
                    DataPoint::from(a.raw_datapoint.clone()).expect("cannot load datapoint")
                }
            }),
        }
    }

    fn store(&mut self, k: Bytes32, datapoint: DataPoint) {
        let a = self.write.get_mut(&k).expect("cannot load from datapoint");
        (*a).raw_datapoint = Vec::from(datapoint);
    }
}

pub(crate) struct SolanaClock {
    current_timestamp: u32,
}

impl SolanaClock {
    pub fn new(current_timestamp: u32) -> Self {
        Self { current_timestamp }
    }
}

impl TimestampChecker for SolanaClock {
    fn current_timestamp(&self) -> u32 {
        self.current_timestamp
    }
}

/// The dummy signature checker for solana. Reason we don't really need signature validation
/// here is because solana has done this for us. The implementation should have asked solana
/// to validate the signatures before hand. All this needs to do is just tracking the number
/// of signatures
pub(crate) struct DummySignatureManger;
impl SignatureManger for DummySignatureManger {
    /// Solana should have already verified the signature for us, return true by default
    fn verify(_key: &[u8], _message: &[u8], _signature: &[u8]) -> bool {
        true
    }
}

pub(crate) fn check_beacon_ids(
    beacon_ids: &[Bytes32],
    pdas: &[Pubkey],
    program_id: &Pubkey,
) -> Result<()> {
    ensure!(
        beacon_ids.len() >= pdas.len(),
        Error::from(ProgramError::from(ERROR_DATA_LENGTH_NOT_MATCH))
    )?;
    let diff = beacon_ids.len() - pdas.len();
    for i in 0..pdas.len() {
        ensure!(
            derive_datapoint_pubkey(&beacon_ids[diff + i], program_id) == pdas[i],
            Error::from(ProgramError::from(ERROR_INVALID_BEACON_ID_KEY))
        )?;
    }
    Ok(())
}

/// Checks the dapis_id passed as parameter is actually derived from beacon_ids.
pub(crate) fn check_dapi_id(dapi_id: &Bytes32, beacon_ids: &[Bytes32]) -> Result<()> {
    ensure!(
        *dapi_id == api3_common::derive_dapi_id(beacon_ids),
        Error::from(ProgramError::from(ERROR_INVALID_DERIVED_DAPI_ID_KEY))
    )?;
    Ok(())
}

/// Checks the dapis_id passed as parameter is actually derived from beacon_ids.
pub(crate) fn check_name_hash(name: &Bytes32, name_hash: &Bytes32) -> Result<()> {
    let derived_hash = keccak_packed(&[Token::FixedBytes(name.to_vec())]);
    ensure!(
        *name_hash == derived_hash,
        Error::from(ProgramError::from(ERROR_INVALID_NAME_HASH))
    )
}

pub(crate) fn check_sys_program(program_id: &Pubkey) -> Result<()> {
    ensure!(
        *program_id == anchor_lang::solana_program::system_program::id(),
        Error::from(ProgramError::from(ERROR_INVALID_SYSTEM_PROGRAM_ID))
    )
}

fn derive_datapoint_pubkey(datapoint_key: &[u8], program_id: &Pubkey) -> Pubkey {
    let (key, _) =
        Pubkey::find_program_address(&[DATAPOINT_SEED.as_bytes(), datapoint_key], program_id);
    key
}

// fn derive_hashname_pubkey(datapoint_key: &[u8], program_id: &Pubkey) -> Pubkey {
//     let (key, _) = Pubkey::find_program_address(
//         &[HASH_NAME_SEED.as_bytes(), datapoint_key],
//         program_id
//     );
//     key
// }
