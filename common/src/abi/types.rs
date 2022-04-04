// construct_uint is giving a lot of clippy warnings. Since most of this file is copied
// over from other crates, it's ok to allow warnings.
#![allow(clippy::ptr_offset_with_cast)]
#![allow(clippy::assign_op_pattern)]
#![allow(clippy::manual_range_contains)]

use uint::construct_uint;

pub type Address = [u8; 20];
pub type FixedBytes = Vec<u8>;
pub type Bytes = Vec<u8>;
pub type Uint = U256;
pub type Word = [u8; 32];

construct_uint! {
    /// 256-bit unsigned integer.
    #[cfg_attr(feature = "scale-info", derive(TypeInfo))]
    pub struct U256(4);
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
    /// String.
    ///
    /// solidity name: string
    /// Encoded in the same way as bytes. Must be utf8 compliant.
    String(String),
}

/// Function and event param types.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamType {
    /// Address.
    Address,
    /// Bytes.
    Bytes,
    /// Unsigned integer.
    Uint(usize),
    /// String.
    String,
    /// Vector of bytes with fixed size.
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

    /// returns whether a ParamType is dynamic
    /// used to decide how the ParamType should be encoded
    pub fn is_dynamic(&self) -> bool {
        matches!(self, ParamType::Bytes | ParamType::String)
    }
}

#[cfg(test)]
mod tests {
    use crate::abi::types::ParamType;

    #[test]
    fn test_is_dynamic() {
        assert!(!ParamType::Address.is_dynamic());
        assert!(ParamType::Bytes.is_dynamic());
        assert!(!ParamType::FixedBytes(32).is_dynamic());
        assert!(!ParamType::Uint(256).is_dynamic());
        assert!(ParamType::String.is_dynamic());
    }
}
