mod adaptor;

use crate::{Bytes, Bytes32};
pub use adaptor::{decode, encode, Address, FixedBytes, Int, ParamType, Token, Uint, U256};
use tiny_keccak::{Hasher, Keccak};

/// Rust implementation of solidity abi.encodePacked(...)
pub fn encode_packed(items: &[Token]) -> (Bytes, String) {
    let res = items.iter().fold(Vec::new(), |mut acc, i| {
        let pack = pack(i);
        acc.push(pack);
        acc
    });
    let res = res.join(&[][..]);
    let hexed = hex::encode(&res);
    (res, hexed)
}

/// Pack a single `Token` into bytes
fn pack(t: &Token) -> Vec<u8> {
    let mut res = Vec::new();
    match t {
        Token::String(s) => res.extend(s.as_bytes()),
        Token::Address(a) => res.extend(a.as_bytes()),
        Token::Uint(n) => {
            let mut v = vec![0u8; 32];
            n.to_big_endian(&mut v);
            res.extend(v);
        }
        Token::Bytes(b) | Token::FixedBytes(b) => res.extend(b),
        _ => {}
    };
    res
}

pub fn keccak256(x: &[u8]) -> Bytes32 {
    let mut keccak = Keccak::v256();
    keccak.update(x);
    let mut out = [0u8; 32];
    keccak.finalize(&mut out);
    out
}

pub fn to_eth_signed_message_hash(s: &[u8]) -> Bytes32 {
    let (bytes, _) = encode_packed(&[
        Token::String("\x19Ethereum Signed Message:\n".parse().unwrap()),
        Token::String(s.len().to_string()),
        Token::Bytes(s.to_vec()),
    ]);
    keccak256(&bytes)
}

#[cfg(feature = "recovery")]
fn public_key_to_address(p: libsecp256k1::PublicKey) -> Address {
    let hash = keccak256(&p.serialize()[1..]);
    Address::from_slice(&hash[12..])
}

#[cfg(feature = "recovery")]
pub fn recover(message: &Bytes32, signature: &[u8; 65]) -> Result<Address, crate::Error> {
    let m = libsecp256k1::Message::parse(message);

    let mut s = [0u8; 64];
    s.copy_from_slice(&signature[..64]);

    let sig = libsecp256k1::Signature::parse_standard(&s)?;
    let i = libsecp256k1::RecoveryId::parse(signature[64] - 27)?;
    let p = libsecp256k1::recover(&m, &sig, &i)?;
    Ok(public_key_to_address(p))
}

#[cfg(test)]
mod tests {
    use crate::abi::{encode_packed, keccak256, Token};
    use crate::abi::{Address, Uint};
    use crate::to_eth_signed_message_hash;
    use hex_literal::hex;

    #[test]
    fn encode_packed_works() {
        let (_, hex_str) = encode_packed(&[Token::String("hello_world".parse().unwrap())]);
        assert_eq!(hex_str, "68656c6c6f5f776f726c64");

        let mut u256 = [0u8; 32];
        u256[0] = 8;
        u256[1] = 10;
        let (_, hex_str) = encode_packed(&[Token::Uint(Uint::from(u256))]);
        assert_eq!(
            hex_str,
            "080a000000000000000000000000000000000000000000000000000000000000"
        );

        let mut h160 = [0u8; 20];
        h160.copy_from_slice(&hex::decode("85B0c8b91707B68C0B23388001B9dC7aab3f6A81").unwrap());
        let (_, hex_str) = encode_packed(&[
            Token::String("hello_world".parse().unwrap()),
            Token::Address(Address::from(h160)),
        ]);
        assert_eq!(
            hex_str,
            "68656c6c6f5f776f726c6485b0c8b91707b68c0b23388001b9dc7aab3f6a81"
        );
    }

    #[test]
    fn keccak_works() {
        let bytes = keccak256(&vec![1, 2, 3]);
        assert_eq!(
            hex::encode(bytes),
            "f1885eda54b7a053318cd41e2093220dab15d65381b1157a3633a83bfd5c9239"
        );
    }

    #[test]
    fn more_complex_test() {
        let mut bytes32 = vec![0; 32];
        bytes32[0] = 8;
        bytes32[1] = 10;

        let mut bytes = vec![0; 36];
        bytes[0] = 18;
        bytes[1] = 120;

        let p1 = Token::FixedBytes(bytes32);
        let p2 = Token::Uint(Uint::from(100));
        let p3 = Token::Bytes(bytes);

        let (b, _) = encode_packed(&[p1, p2, p3]);
        assert_eq!(b, hex!("080a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000064127800000000000000000000000000000000000000000000000000000000000000000000"));
        assert_eq!(
            keccak256(&b),
            hex!("f3f0d971e5307ceec7ca3d9b762780778c4efe8383f0e17015a2cf8ac8dbc179")
        );
    }

    #[test]
    fn to_eth_signed_message_hash_works() {
        let mut bytes32 = vec![0; 32];
        bytes32[0] = 8;
        bytes32[1] = 10;

        let mut bytes = vec![0; 36];
        bytes[0] = 18;
        bytes[1] = 120;

        let p1 = Token::FixedBytes(bytes32);
        let p2 = Token::Uint(Uint::from(100));
        let p3 = Token::Bytes(bytes);

        let (b, _) = encode_packed(&[p1, p2, p3]);
        assert_eq!(b, hex!("080a0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000064127800000000000000000000000000000000000000000000000000000000000000000000"));

        let b = keccak256(&b);
        assert_eq!(
            b,
            hex!("f3f0d971e5307ceec7ca3d9b762780778c4efe8383f0e17015a2cf8ac8dbc179")
        );

        let b = to_eth_signed_message_hash(&b);
        assert_eq!(
            b,
            hex!("ff0d3be602bd7ed7c0454766464e6a1a9130a63cd505e629ae133db5c3b9f149")
        );
    }

    #[test]
    #[cfg(feature = "recovery")]
    fn verify_works() {
        let mut bytes = vec![0; 36];
        bytes[0] = 18;
        bytes[1] = 120;

        let message = to_eth_signed_message_hash(&bytes);
        let signature = hex::decode("c01673d51c5e9276a380959a147a29b56e9ab47ef9fd0183c1f1155b8a1ac094571be063ea415c293bd986f31eb31faa790858bf48260157b2978b6edadd07a91b").unwrap();

        let mut s = [0u8; 65];
        s.copy_from_slice(&signature);

        let pubkey = crate::recover(&message, &s).unwrap();
        let address = hex::decode("65B0c8b91707B68C0B23388001B9dC7aab3f6A81").unwrap();
        assert_eq!(pubkey.as_bytes().to_vec(), address);
    }
}
