mod abi;
mod access;
mod access_control_registry;
mod agg;
mod beacon;
mod datapoint;
mod error;
pub mod types;
pub mod util;
mod whitelist;

pub use abi::*;
pub use access::*;
pub use agg::Aggregator;
pub use beacon::*;
pub use datapoint::DataPoint;
pub use error::Error;
pub use util::*;
pub use whitelist::Whitelist;

pub type Bytes = Vec<u8>;
pub type Bytes32 = [u8; 32];
pub const BYTES32_ZERO: Bytes32 = [0u8; 32];

#[macro_export]
macro_rules! ensure {
    ( $x:expr, $y:expr ) => {{
        if !$x {
            Err($y)
        } else {
            Ok(())
        }
    }};
}

/// Checks if the address is zero
pub trait Zero {
    fn is_zero(&self) -> bool;
}

impl Zero for Bytes32 {
    fn is_zero(&self) -> bool {
        (*self) == BYTES32_ZERO
    }
}
