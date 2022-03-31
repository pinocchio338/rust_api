//! ABI encoder.

use crate::abi::{
    types::{Bytes, Word},
    Token,
};

/// Converts a u32 to a right aligned array of 32 bytes.
pub fn pad_u32(value: u32) -> Word {
    let mut padded = [0u8; 32];
    padded[28..32].copy_from_slice(&value.to_be_bytes());
    padded
}

fn pad_bytes(bytes: &[u8]) -> Vec<Word> {
    let mut result = vec![pad_u32(bytes.len() as u32)];
    result.extend(pad_fixed_bytes(bytes));
    result
}

fn pad_fixed_bytes(bytes: &[u8]) -> Vec<Word> {
    let len = (bytes.len() + 31) / 32;
    let mut result = Vec::with_capacity(len);
    for i in 0..len {
        let mut padded = [0u8; 32];

        let to_copy = match i == len - 1 {
            false => 32,
            true => match bytes.len() % 32 {
                0 => 32,
                x => x,
            },
        };

        let offset = 32 * i;
        padded[..to_copy].copy_from_slice(&bytes[offset..offset + to_copy]);
        result.push(padded);
    }

    result
}

#[derive(Debug)]
enum Mediate {
    Raw(Vec<Word>),
    Prefixed(Vec<Word>),
}

impl Mediate {
    fn head_len(&self) -> u32 {
        match *self {
            Mediate::Raw(ref raw) => 32 * raw.len() as u32,
            Mediate::Prefixed(_) => 32,
        }
    }

    fn tail_len(&self) -> u32 {
        match *self {
            Mediate::Raw(_) => 0,
            Mediate::Prefixed(ref pre) => pre.len() as u32 * 32,
        }
    }

    fn head(&self, suffix_offset: u32) -> Vec<Word> {
        match *self {
            Mediate::Raw(ref raw) => raw.clone(),
            Mediate::Prefixed(_) => vec![pad_u32(suffix_offset)],
        }
    }

    fn tail(&self) -> Vec<Word> {
        match *self {
            Mediate::Raw(_) => vec![],
            Mediate::Prefixed(ref raw) => raw.clone(),
        }
    }
}

fn encode_head_tail(mediates: &[Mediate]) -> Vec<Word> {
    let heads_len = mediates.iter().fold(0, |acc, m| acc + m.head_len());

    let (mut result, len) = mediates.iter().fold(
        (Vec::with_capacity(heads_len as usize), heads_len),
        |(mut acc, offset), m| {
            acc.extend(m.head(offset));
            (acc, offset + m.tail_len())
        },
    );

    let tails = mediates.iter().fold(
        Vec::with_capacity((len - heads_len) as usize),
        |mut acc, m| {
            acc.extend(m.tail());
            acc
        },
    );

    result.extend(tails);
    result
}

/// Encodes vector of tokens into ABI compliant vector of bytes.
pub fn encode(tokens: &[Token]) -> Bytes {
    let mediates = &tokens.iter().map(encode_token).collect::<Vec<_>>();

    encode_head_tail(mediates)
        .iter()
        .flat_map(|word| word.to_vec())
        .collect()
}

fn encode_token(token: &Token) -> Mediate {
    match token {
        Token::Address(ref address) => {
            let mut padded = [0u8; 32];
            padded[12..].copy_from_slice(address);
            Mediate::Raw(vec![padded])
        }
        Token::Bytes(ref bytes) => Mediate::Prefixed(pad_bytes(bytes)),
        Token::String(ref s) => Mediate::Prefixed(pad_bytes(s.as_bytes())),
        Token::FixedBytes(ref bytes) => Mediate::Raw(pad_fixed_bytes(bytes)),
        Token::Uint(uint) => Mediate::Raw(vec![uint.clone().into()]),
    }
}

#[cfg(test)]
mod tests {
    use crate::abi::encode::{encode, pad_u32};
    use crate::abi::Token;
    use hex_literal::hex;

    #[test]
    fn encode_address() {
        let address = Token::Address([0x11u8; 20].into());
        let encoded = encode(&[address]);
        let expected = hex!("0000000000000000000000001111111111111111111111111111111111111111");
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_two_addresses() {
        let address1 = Token::Address([0x11u8; 20].into());
        let address2 = Token::Address([0x22u8; 20].into());
        let encoded = encode(&[address1, address2]);
        let expected = hex!(
            "
			0000000000000000000000001111111111111111111111111111111111111111
			0000000000000000000000002222222222222222222222222222222222222222
		"
        )
        .to_vec();
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_bytes() {
        let bytes = Token::Bytes(vec![0x12, 0x34]);
        let encoded = encode(&[bytes]);
        let expected = hex!(
            "
			0000000000000000000000000000000000000000000000000000000000000020
			0000000000000000000000000000000000000000000000000000000000000002
			1234000000000000000000000000000000000000000000000000000000000000
		"
        )
        .to_vec();
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_fixed_bytes() {
        let bytes = Token::FixedBytes(vec![0x12, 0x34]);
        let encoded = encode(&[bytes]);
        let expected = hex!("1234000000000000000000000000000000000000000000000000000000000000");
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_string() {
        let s = Token::String("gavofyork".to_owned());
        let encoded = encode(&[s]);
        let expected = hex!(
            "
			0000000000000000000000000000000000000000000000000000000000000020
			0000000000000000000000000000000000000000000000000000000000000009
			6761766f66796f726b0000000000000000000000000000000000000000000000
		"
        )
        .to_vec();
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_bytes2() {
        let bytes = Token::Bytes(
            hex!("10000000000000000000000000000000000000000000000000000000000002").to_vec(),
        );
        let encoded = encode(&[bytes]);
        let expected = hex!(
            "
			0000000000000000000000000000000000000000000000000000000000000020
			000000000000000000000000000000000000000000000000000000000000001f
			1000000000000000000000000000000000000000000000000000000000000200
		"
        )
        .to_vec();
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_bytes3() {
        let bytes = Token::Bytes(
            hex!(
                "
			1000000000000000000000000000000000000000000000000000000000000000
			1000000000000000000000000000000000000000000000000000000000000000
		"
            )
            .to_vec(),
        );
        let encoded = encode(&[bytes]);
        let expected = hex!(
            "
			0000000000000000000000000000000000000000000000000000000000000020
			0000000000000000000000000000000000000000000000000000000000000040
			1000000000000000000000000000000000000000000000000000000000000000
			1000000000000000000000000000000000000000000000000000000000000000
		"
        )
        .to_vec();
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_two_bytes() {
        let bytes1 = Token::Bytes(
            hex!("10000000000000000000000000000000000000000000000000000000000002").to_vec(),
        );
        let bytes2 = Token::Bytes(
            hex!("0010000000000000000000000000000000000000000000000000000000000002").to_vec(),
        );
        let encoded = encode(&[bytes1, bytes2]);
        let expected = hex!(
            "
			0000000000000000000000000000000000000000000000000000000000000040
			0000000000000000000000000000000000000000000000000000000000000080
			000000000000000000000000000000000000000000000000000000000000001f
			1000000000000000000000000000000000000000000000000000000000000200
			0000000000000000000000000000000000000000000000000000000000000020
			0010000000000000000000000000000000000000000000000000000000000002
		"
        )
        .to_vec();
        assert_eq!(encoded, expected);
    }

    #[test]
    fn encode_uint() {
        let mut uint = [0u8; 32];
        uint[31] = 4;
        let encoded = encode(&[Token::Uint(uint.into())]);
        let expected = hex!("0000000000000000000000000000000000000000000000000000000000000004");
        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_pad_u32() {
        // this will fail if endianess is not supported
        assert_eq!(pad_u32(0x1)[31], 1);
        assert_eq!(pad_u32(0x100)[30], 1);
    }
}
