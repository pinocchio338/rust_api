mod abi;
mod beacon;
mod datapoint;
mod error;
mod agg;
mod util;

pub use abi::*;
pub use beacon::*;
pub use datapoint::DataPoint;
pub use error::Error;
pub use agg::Aggregator;
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
