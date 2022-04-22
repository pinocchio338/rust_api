use api3_common::Zero;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::AccountId;
use serde::{Deserialize, Serialize};
use std::io;

/// Address is an alias to H160, which is [u8;20]
#[derive(Serialize, Deserialize, BorshSerialize, BorshDeserialize)]
pub struct Address(AccountId);

impl Address {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl Zero for Address {
    fn is_zero(&self) -> bool {
        todo!();
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<&str> for Address {
    fn from(value: &str) -> Self {
        Address(value.to_string())
    }
}
