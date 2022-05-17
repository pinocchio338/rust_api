#![allow(clippy::assign_op_pattern)]
#![allow(clippy::ptr_offset_with_cast)]

use crate::Bytes;
use borsh::{BorshDeserialize, BorshSerialize};
use std::io;
use uint::construct_uint;

pub type Address = [u8; 20];
pub type FixedBytes = Vec<u8>;
pub type Uint = U256;
pub type Word = [u8; 32];
pub type Int = Uint;

construct_uint! {
    pub struct U256(4);
}

impl BorshDeserialize for U256 {
    fn deserialize(bytes: &mut &[u8]) -> Result<Self, io::Error> {
        let values: [u8; 32] = BorshDeserialize::deserialize(bytes)?;
        Ok(U256::from_big_endian(&values))
    }
}

impl BorshSerialize for U256 {
    fn serialize<W>(&self, writer: &mut W) -> Result<(), io::Error>
    where
        W: io::Write,
    {
        let mut v = [0u8; 32];
        self.to_big_endian(&mut v);
        BorshSerialize::serialize(&v, writer)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    /// Address.
    ///
    /// solidity name: address
    /// Encoded to left padded [0u8; 32].
    Address(Address),
    /// Vector of bytes with known size.
    ///
    /// solidity name eg.: bytes8, bytes32, bytes64, bytes1024
    /// Encoded to right padded [0u8; ((N + 31) / 32) * 32].
    FixedBytes(FixedBytes),
    /// Vector of bytes of unknown size.
    ///
    /// solidity name: bytes
    /// Encoded in two parts.
    /// Init part: offset of 'closing part`.
    /// Closing part: encoded length followed by encoded right padded bytes.
    Bytes(Bytes),
    /// Unsigned integer.
    ///
    /// solidity name: uint
    Uint(Uint),
    /// Signed integer.
    ///
    /// solidity name: int
    Int(Int),
    /// String.
    ///
    /// solidity name: string
    /// Encoded in the same way as bytes. Must be utf8 compliant.
    String(String),
}

/// Function and event param types.
#[derive(PartialEq)]
pub enum ParamType {
    /// Address
    Address,
    /// Bytes
    Bytes,
    /// Unsigned integer
    Uint(usize),
    /// Signed integer
    Int(usize),
    /// String
    String,
    /// Vector of bytes with fixed size
    FixedBytes(usize),
}

impl ParamType {
    /// returns whether a zero length byte slice (`0x`) is
    /// a valid encoded form of this param type
    pub fn is_empty_bytes_valid_encoding(&self) -> bool {
        match self {
            ParamType::FixedBytes(len) => *len == 0,
            _ => false,
        }
    }
}
