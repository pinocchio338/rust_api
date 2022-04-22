use borsh::{self, BorshDeserialize, BorshSerialize};
use derive_more::{
    Add, AddAssign, Display, Div, DivAssign, From, Into, Mul, MulAssign, Sub, SubAssign,
};
use serde::{Deserialize, Serialize};
use std::io;

/// This needs to be wrapped here, otherwise
/// There is no way for use to implement Borsh serde for U256 due to the fact
/// that both type and trait are foreign
#[derive(
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Default,
    From,
    Into,
    Display,
    Add,
    AddAssign,
    Sub,
    SubAssign,
    Div,
    DivAssign,
    Mul,
    MulAssign,
    Serialize,
    Deserialize,
)]
pub struct U256(pub crate::abi::U256);

impl BorshDeserialize for U256 {
    fn deserialize(bytes: &mut &[u8]) -> Result<Self, io::Error> {
        let values: [u8; 32] = BorshDeserialize::deserialize(bytes)?;
        Ok(U256(crate::abi::U256::from_big_endian(&values)))
    }
}

impl BorshSerialize for U256 {
    fn serialize<W>(&self, writer: &mut W) -> Result<(), io::Error>
    where
        W: io::Write,
    {
        let mut v = [0u8; 32];
        self.0.to_big_endian(&mut v);
        BorshSerialize::serialize(&v, writer)
    }
}

macro_rules! impl_u256 {
    ($($t: ty;)*) => {
        $(
            impl From<$t> for U256 {
                fn from(v: $t) -> Self {
                    U256(crate::abi::U256::from(v))
                }
            }
        )*
    };
}

impl_u256!(i32; i64; isize; i128; u32; u64; usize; u128;);

impl U256 {
    pub fn as_u32(&self) -> u32 {
        self.0.as_u32()
    }
}
