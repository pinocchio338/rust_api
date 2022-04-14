mod abi;
mod access;
mod agg;
mod beacon;
mod datapoint;
mod error;
pub mod types;
mod util;
mod whitelist;

pub use abi::*;
pub use agg::Aggregator;
pub use beacon::*;
pub use datapoint::DataPoint;
pub use error::Error;
pub use util::*;

pub type Bytes = Vec<u8>;
pub type Bytes32 = [u8; 32];

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

pub trait Empty {
    fn is_empty(&self) -> bool;
}
