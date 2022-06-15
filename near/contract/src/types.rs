use api3_common::abi::{Int, U256};
use api3_common::{Bytes32, DataPoint, Zero};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize, PartialEq, Clone, Default)]
pub struct Address(pub Bytes32);

impl From<Address> for Bytes32 {
    fn from(a: Address) -> Self {
        a.0
    }
}

impl Zero for Address {
    fn is_zero(&self) -> bool {
        self.0 == Bytes32::default()
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone)]
pub(crate) struct NearDataPoint {
    pub value: U256,
    pub timestamp: u32,
}

impl NearDataPoint {
    pub fn new(value: U256, timestamp: u32) -> Self {
        NearDataPoint { value, timestamp }
    }
}

impl From<NearDataPoint> for DataPoint {
    fn from(t: NearDataPoint) -> Self {
        let mut v = [0u8; 32];
        t.value.to_big_endian(&mut v);
        DataPoint::new(Int::from_big_endian(&v), t.timestamp)
    }
}

impl From<DataPoint> for NearDataPoint {
    fn from(t: DataPoint) -> Self {
        let mut v = [0u8; 32];
        t.value.to_big_endian(&mut v);
        NearDataPoint::new(U256::from_big_endian(&v), t.timestamp)
    }
}
