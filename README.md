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

# Near
## Prerequisite
Read up on Near from these links:
- Get started: `https://docs.near.org/docs/develop/contracts/overview`
- Create account: `https://docs.near.org/docs/develop/basics/create-account#creating-a-testnet-account`
- Cross contract call: `https://docs.near.org/docs/tutorials/contracts/xcc-receipts`
- End to end test: `https://docs.near.org/docs/develop/contracts/rust/testing-rust-contracts#end-to-end-tests`
After reading the above, you should be able to know:
- The basic syntax of writing a Near contract using rust
- How to create test accounts
- How and why we need cross contract call
- How end to end test with javascript is written and tested

## Dev
To setup the dev env, checkout the dockerfile in `.devcontainer/Near-Dockerfile`, build and launch the docker. Alternatively,
we recommend you use vscode remote docker plugin, you can open this folder in remote docker, then you can have the same dev 
env.
Once in the docker, you can follow the subsection accordingly.

### Compile
- go to the near contract folder: `cd near/contract`
- compile: `cargo build --target wasm32-unknown-unknown --release`
Once done you should be able to see, relative to the repo root folder, `target/wasm32-unknown-unknown/release/dapi_server.wasm`.

### Create test accounts
You need to create 3 accounts for testing:
```
CONTRACT_ACCOUNT: the account for the dapiServer contract.
ADMIN_ACCOUNT: the default admin of the contract.
USER_ACCOUNT: test util account, mainly for reading data points with unlimited access for data verification.
```
Now go to near testnet and create the above accounts, you can choose your own names. Remember to define the above env variables with the account 
names, i.e. for our dev env, it's:
```
export CONTRACT_ACCOUNT=dapi-contract.testnet
export ADMIN_ACCOUNT=mocha-test.testnet
export USER_ACCOUNT=user-test.testnet
```

### Login on CLI
Once the acconts are created, you need to login from CLI:
```
near login --account-id ${CONTRACT_ACCOUNT}
near login --account-id ${ADMIN_ACCOUNT}
near login --account-id ${USER_ACCOUNT}
```

### Deploy the contracts
In the root folder, deploy the `api3-contract` using:
```
near deploy --wasmFile ./target/wasm32-unknown-unknown/release/dapi_server.wasm --contractName=${CONTRACT_ACCOUNT} --keyPath=/home/dev/.near-credentials/testnet/${CONTRACT_ACCOUNT}.json
```

### Run tests
The tests are located in `near/client-test`, the `main` for tests is `near/client-test/src/test.js`.
Locate the DEFINITION of `isInitialized`, it tells the program whether to initialize the contract. If you are running the first time, 
ensure this is `false`, else you can mark this as `true`.
Most of the tests are commented off by default, if you want to test specific parts, uncomment the section.
To run the test: `cd near/client-test && yarn jest`

### Clean up
To clean up, just delete the accounts using `near delete ... ...`. See `https://docs.near.org/docs/tools/near-cli#near-delete`.