use borsh::{self, BorshDeserialize, BorshSerialize};
use derive_more::{Add, AddAssign, Display, From, Into, Sub, SubAssign};
use serde::{Deserialize, Serialize};
use std::io;

#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    PartialOrd,
    Default,
    From,
    Into,
    Display,
    Add,
    AddAssign,
    Sub,
    SubAssign,
)]
pub struct U256(ethereum_types::U256);

impl BorshDeserialize for U256 {
    fn deserialize(bytes: &mut &[u8]) -> Result<Self, io::Error> {
        let values: [u64; 4] = BorshDeserialize::deserialize(bytes)?;
        Ok(U256(ethereum_types::U256(values)))
    }
}

impl BorshSerialize for U256 {
    fn serialize<W>(&self, writer: &mut W) -> Result<(), io::Error>
    where
        W: io::Write,
    {
        BorshSerialize::serialize(&self.0 .0, writer)
    }
}

impl From<u128> for U256 {
    fn from(v: u128) -> Self {
        U256(ethereum_types::U256::from(v))
    }
}

/// Address is an alias to H160, which is [u8;20]
#[derive(Serialize, Deserialize)]
pub struct Address(ethereum_types::Address);

impl BorshDeserialize for Address {
    fn deserialize(bytes: &mut &[u8]) -> Result<Self, io::Error> {
        let values: [u8; 20] = BorshDeserialize::deserialize(bytes)?;
        Ok(Address(ethereum_types::Address::from(values)))
    }
}

impl BorshSerialize for Address {
    fn serialize<W>(&self, writer: &mut W) -> Result<(), io::Error>
    where
        W: io::Write,
    {
        BorshSerialize::serialize(&self.0 .0, writer)
    }
}

impl From<[u8; 20]> for Address {
    fn from(bytes: [u8; 20]) -> Self {
        Address(ethereum_types::Address::from(bytes))
    }
}

#[test]
fn serialization() {
    let mut buffer = vec![];
    let v = U256::from(u128::MAX);
    dbg!(&v);
    v.serialize(&mut buffer).unwrap();
    dbg!(&buffer);

    let uv = U256::try_from_slice(&mut buffer);
    dbg!(&uv);
    assert_eq!(v, uv.unwrap());
}
