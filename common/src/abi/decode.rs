//! ABI decoder.

use crate::abi::{ParamType, Token, Word};
use crate::Error;

#[derive(Debug)]
struct DecodeResult {
    token: Token,
    new_offset: usize,
}

fn as_usize(slice: &Word) -> Result<usize, Error> {
    if !slice[..28].iter().all(|x| *x == 0) {
        return Err(Error::InvalidData);
    }

    let result = ((slice[28] as usize) << 24)
        + ((slice[29] as usize) << 16)
        + ((slice[30] as usize) << 8)
        + (slice[31] as usize);

    Ok(result)
}

/// Decodes ABI compliant vector of bytes into vector of tokens described by types param.
pub fn decode(types: &[ParamType], data: &[u8]) -> Result<Vec<Token>, Error> {
    let is_empty_bytes_valid_encoding = types.iter().all(|t| t.is_empty_bytes_valid_encoding());
    if !is_empty_bytes_valid_encoding && data.is_empty() {
        return Err(Error::InvalidName(
            "please ensure the contract and method you're calling exist! \
			 failed to decode empty bytes. if you're using jsonrpc this is \
			 likely due to jsonrpc returning `0x` in case contract or method \
			 don't exist"
                .into(),
        ));
    }

    let mut tokens = vec![];
    let mut offset = 0;

    for param in types {
        let res = decode_param(param, data, offset)?;
        offset = res.new_offset;
        tokens.push(res.token);
    }

    Ok(tokens)
}

fn peek(data: &[u8], offset: usize, len: usize) -> Result<&[u8], Error> {
    if offset + len > data.len() {
        Err(Error::InvalidData)
    } else {
        Ok(&data[offset..(offset + len)])
    }
}

fn peek_32_bytes(data: &[u8], offset: usize) -> Result<Word, Error> {
    peek(data, offset, 32).map(|x| {
        let mut out: Word = [0u8; 32];
        out.copy_from_slice(&x[0..32]);
        out
    })
}

fn take_bytes(data: &[u8], offset: usize, len: usize) -> Result<Vec<u8>, Error> {
    if offset + len > data.len() {
        Err(Error::InvalidData)
    } else {
        Ok((data[offset..(offset + len)]).to_vec())
    }
}

fn decode_param(param: &ParamType, data: &[u8], offset: usize) -> Result<DecodeResult, Error> {
    match *param {
        ParamType::Address => {
            let slice = peek_32_bytes(data, offset)?;
            let mut address = [0u8; 20];
            address.copy_from_slice(&slice[12..]);
            let result = DecodeResult {
                token: Token::Address(address),
                new_offset: offset + 32,
            };
            Ok(result)
        }
        ParamType::Uint(_) => {
            let slice = peek_32_bytes(data, offset)?;
            let result = DecodeResult {
                token: Token::Uint(slice.into()),
                new_offset: offset + 32,
            };
            Ok(result)
        }
        ParamType::Int(_) => {
            let slice = peek_32_bytes(data, offset)?;
            let result = DecodeResult {
                token: Token::Int(slice.into()),
                new_offset: offset + 32,
            };
            Ok(result)
        }
        ParamType::FixedBytes(len) => {
            // FixedBytes is anything from bytes1 to bytes32. These values
            // are padded with trailing zeros to fill 32 bytes.
            let bytes = take_bytes(data, offset, len)?;
            let result = DecodeResult {
                token: Token::FixedBytes(bytes),
                new_offset: offset + 32,
            };
            Ok(result)
        }
        ParamType::Bytes => {
            let dynamic_offset = as_usize(&peek_32_bytes(data, offset)?)?;
            let len = as_usize(&peek_32_bytes(data, dynamic_offset)?)?;
            let bytes = take_bytes(data, dynamic_offset + 32, len)?;
            let result = DecodeResult {
                token: Token::Bytes(bytes),
                new_offset: offset + 32,
            };
            Ok(result)
        }
        ParamType::String => {
            let dynamic_offset = as_usize(&peek_32_bytes(data, offset)?)?;
            let len = as_usize(&peek_32_bytes(data, dynamic_offset)?)?;
            let bytes = take_bytes(data, dynamic_offset + 32, len)?;
            let result = DecodeResult {
                // NOTE: We're decoding strings using lossy UTF-8 decoding to
                // prevent invalid strings written into contracts by either users or
                // Solidity bugs from causing graph-node to fail decoding event
                // data.
                token: Token::String(String::from_utf8_lossy(&*bytes).into()),
                new_offset: offset + 32,
            };
            Ok(result)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::abi::decode::decode;
    use crate::abi::types::ParamType;
    use crate::abi::{Token, Uint};
    use hex_literal::hex;

    #[test]
    fn decode_from_empty_byte_slice() {
        // these can NOT be decoded from empty byte slice
        assert!(decode(&[ParamType::Address], &[]).is_err());
        assert!(decode(&[ParamType::Bytes], &[]).is_err());
        assert!(decode(&[ParamType::String], &[]).is_err());
        assert!(decode(&[ParamType::FixedBytes(1)], &[]).is_err());
        // these are the only ones that can be decoded from empty byte slice
        assert!(decode(&[ParamType::FixedBytes(0)], &[]).is_ok());
    }

    #[test]
    fn decode_data_with_size_that_is_not_a_multiple_of_32() {
        let encoded = hex!(
            "
            0000000000000000000000000000000000000000000000000000000000000000
            00000000000000000000000000000000000000000000000000000000000000a0
            0000000000000000000000000000000000000000000000000000000000000152
            0000000000000000000000000000000000000000000000000000000000000001
            000000000000000000000000000000000000000000000000000000000054840d
            0000000000000000000000000000000000000000000000000000000000000092
            3132323033393637623533326130633134633938306235616566666231373034
            3862646661656632633239336139353039663038656233633662306635663866
            3039343265376239636337366361353163636132366365353436393230343438
            6533303866646136383730623565326165313261323430396439343264653432
            3831313350373230703330667073313678390000000000000000000000000000
            0000000000000000000000000000000000103933633731376537633061363531
            3761
        "
        );

        assert_eq!(
            decode(
                &[
                    ParamType::Uint(256),
                    ParamType::String,
                    ParamType::String,
                    ParamType::Uint(256),
                    ParamType::Uint(256),
                ],
                &encoded,
            ).unwrap(),
            &[
                Token::Uint(Uint::from(0u128)),
                Token::String(String::from("12203967b532a0c14c980b5aeffb17048bdfaef2c293a9509f08eb3c6b0f5f8f0942e7b9cc76ca51cca26ce546920448e308fda6870b5e2ae12a2409d942de428113P720p30fps16x9")),
                Token::String(String::from("93c717e7c0a6517a")),
                Token::Uint(Uint::from(1u128)),
                Token::Uint(Uint::from(5538829u128))
            ]
        );
    }

    #[test]
    fn decode_after_fixed_bytes_with_less_than_32_bytes() {
        let encoded = hex!(
            "
			0000000000000000000000008497afefdc5ac170a664a231f6efb25526ef813f
			0000000000000000000000000000000000000000000000000000000000000000
			0000000000000000000000000000000000000000000000000000000000000000
			0000000000000000000000000000000000000000000000000000000000000080
			000000000000000000000000000000000000000000000000000000000000000a
			3078303030303030314600000000000000000000000000000000000000000000
		"
        );

        assert_eq!(
            decode(
                &[
                    ParamType::Address,
                    ParamType::FixedBytes(32),
                    ParamType::FixedBytes(4),
                    ParamType::String,
                ],
                &encoded,
            )
            .unwrap(),
            &[
                Token::Address(hex!("8497afefdc5ac170a664a231f6efb25526ef813f").into()),
                Token::FixedBytes([0u8; 32].to_vec()),
                Token::FixedBytes([0u8; 4].to_vec()),
                Token::String("0x0000001F".into()),
            ]
        )
    }

    #[test]
    fn decode_broken_utf8() {
        let encoded = hex!(
            "
			0000000000000000000000000000000000000000000000000000000000000020
			0000000000000000000000000000000000000000000000000000000000000004
			e4b88de500000000000000000000000000000000000000000000000000000000
        "
        );

        assert_eq!(
            decode(&[ParamType::String,], &encoded).unwrap(),
            &[Token::String("不�".into())]
        );
    }
}
