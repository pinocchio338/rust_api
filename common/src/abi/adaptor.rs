use crate::Error;
use ethabi::decode as eth_decode;
pub use ethabi::{encode, Address, FixedBytes, Int, ParamType, Token, Uint, Uint as U256};

impl From<ethabi::Error> for Error {
    fn from(e: ethabi::Error) -> Self {
        Error::EthAbiError(e)
    }
}

pub fn decode(types: &[ParamType], data: &[u8]) -> Result<Vec<Token>, Error> {
    let v = eth_decode(types, data)?;
    Ok(v)
}
