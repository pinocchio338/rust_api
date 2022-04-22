use crate::{encode_packed, keccak256, Bytes32, Token};
pub use median::median;
pub use median::median_wrapped_u256;
pub use sort::sort;

mod median;
mod sort;

pub fn keccak_packed(tokens: &[Token]) -> Bytes32 {
    let (encoded, _) = encode_packed(tokens);
    keccak256(&encoded)
}
