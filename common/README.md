# API3 Common Crate
This is the common crate for porting solidity API3 contracts to Rust based chains. As for different chains, the same processing 
logic would be applied, it is natural to abstract common processes.
The main design is as follows:
* Common data types
  * DataPoint: The datapoint struct used in the original solidity contracts.
  * Role: Some of the roles known at dev time are modelled using enum
* Common methods: In `common/src/beacon.rs`, it contains all the methods used in the original `DapiServer.sol`. 
All the methods are implemented the same as in the solidity contracts. To ensure everything works in the respective chains, 
the chain specific operations are abstracted into traits so that each chain could have its own implementation. The following traits 
are implemented:
  * Storage<T>: `common/src/beacon::Storage`handles the load/save of item type T in the chain
  * Whitelist: `common/src/whitelist.rs:20` handles the whitelist functions in the specific chain
  * AccessControlRegistry: `common/src/access::AccessControlRegistry` handles the access control related function in the specific chain
  * SignatureManger: `common/src/beacon::SignatureManger` handles the onchain signature verification