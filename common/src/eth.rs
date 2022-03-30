use ethabi::Token;
use tiny_keccak::{Hasher, Keccak};
use crate::{Bytes, Bytes32};

pub fn keccak(x: &[u8]) -> Bytes32 {
    let mut keccak = Keccak::v256();
    keccak.update(x);
    let mut out = [0u8; 32];
    keccak.finalize(&mut out);
    out
}

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
        },
        Token::Bytes(b) => res.extend(b),
        _ => panic!("not supported yet"),
    };
    return res;
}

#[cfg(test)]
mod tests {
    use ethabi::{Address, Token, Uint};
    use crate::{encode_packed, keccak};

    #[test]
    fn encode_packed_works() {
        let (_, hex_str) = encode_packed(&[Token::String("hello_world".parse().unwrap())]);
        assert_eq!(hex_str, "68656c6c6f5f776f726c64");

        let mut u256 = [0u8; 32];
        u256[0] = 8;
        u256[1] = 10;
        let (_, hex_str) = encode_packed(&[Token::Uint(Uint::from(u256))]);
        assert_eq!(hex_str, "080a000000000000000000000000000000000000000000000000000000000000");

        let mut h160 = [0u8; 20];
        h160.copy_from_slice(&hex::decode("85B0c8b91707B68C0B23388001B9dC7aab3f6A81").unwrap());
        let (_, hex_str) = encode_packed(&[
            Token::String("hello_world".parse().unwrap()),
            Token::Address(Address::from(h160))
        ]);
        assert_eq!(hex_str, "68656c6c6f5f776f726c6485b0c8b91707b68c0b23388001b9dc7aab3f6a81");
    }

    #[test]
    fn keccak_works() {
        let bytes = keccak(&vec![1, 2, 3]);
        assert_eq!(hex::encode(bytes), "f1885eda54b7a053318cd41e2093220dab15d65381b1157a3633a83bfd5c9239");
    }
}