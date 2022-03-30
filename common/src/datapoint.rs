use crate::{error, Uint256};

/// The data point struct in the original API3 beacon server contract
#[derive(Clone)]
pub struct DataPoint {
    value: Uint256,
    timestamp: u32
}

impl DataPoint {
    /// Len of the data point as bytes, value is 32 bytes and timestamp is 4 bytes
    const LEN: usize = 36;

    pub fn new(value: Uint256, timestamp: u32) -> Self {
        Self { value, timestamp }
    }

    pub fn from(raw: Vec<u8>) -> Result<Self, error::Error> {
        if raw.len() != Self::LEN {
            Err(error::Error::CannotDeserializeDataPoint)
        } else {
            let mut value = Uint256::default();
            value.copy_from_slice(&raw[0..32]);
            Ok(Self {
                value,
                timestamp: u32::from_be_bytes([raw[32], raw[33], raw[34], raw[35]])
            })
        }
    }
}

impl Into<Vec<u8>> for DataPoint {
    fn into(self) -> Vec<u8> {
        let mut v = vec![0u8; DataPoint::LEN];
        v[0..32].copy_from_slice(&self.value);
        v[32..].copy_from_slice(&self.timestamp.to_be_bytes());
        v
    }
}
