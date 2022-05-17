# API3-Rust
This is the repo for RUST implementation of API3's Beacon Server

# Common
Common package used for all the subsequent chain implementations.
To run all test
```
cargo test --all
```

# Solana
Read up on anchors: `https://project-serum.github.io/anchor/`
To build the solana code, do the following:
```
cd solana/beacon-server
anchor build

# make sure you are not in a browser setting with 
export BROWSER=""
solana-keygen new
anchor test
```

TODO:
- Check gas fee
- Provide devnet script
- Solana wallet from seeds