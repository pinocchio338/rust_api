/// derived from NearWhitelist.sol
///
/// @title Contract that implements temporary and permanent whitelists for
/// multiple services identified with a hash
/// @notice This contract implements two kinds of whitelisting:
///   (1) Temporary, ends when the expiration timestamp is in the past
///   (2) Indefinite, ends when the indefinite whitelist count is zero
/// Multiple senders can indefinitely whitelist/unwhitelist independently. The
/// user will be considered whitelisted as long as there is at least one active
/// indefinite whitelisting.
/// @dev The interface of this contract is not implemented. It should be
/// inherited and its functions should be exposed with a sort of an
/// authorization scheme.
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::BorshStorageKey;
use near_sdk::{env, near_bindgen};
use std::io;

use crate::ensure;
use crate::Address;
use api3_common::types::U256;
use api3_common::Bytes32;
use api3_common::Error;
use api3_common::Whitelist;

near_sdk::setup_alloc!();

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Service,
    UserWhitelist,
    ServiceWithSetter,
    UserSetterIndefiniteWhitelist,
}

//#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Default)]
struct WhitelistStatus {
    expiration_timestamp: u64,
    /// orignally uint192, that is u128 and u64 combined
    indefinite_whitelist_count: U256,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
struct NearWhitelist {
    service_id_to_user_to_whitelist_status: LookupMap<Bytes32, LookupMap<Address, WhitelistStatus>>,
    service_id_to_user_to_setter_to_indefinite_whitelist_status:
        LookupMap<Bytes32, LookupMap<Address, LookupMap<Address, bool>>>,
}

impl Default for NearWhitelist {
    fn default() -> Self {
        Self {
            service_id_to_user_to_whitelist_status: LookupMap::new(StorageKeys::Service),
            service_id_to_user_to_setter_to_indefinite_whitelist_status: LookupMap::new(
                StorageKeys::ServiceWithSetter,
            ),
        }
    }
}

impl NearWhitelist {
    /// ensure that the service_id is in the lookup map
    fn ensure_service_exist(&mut self, service_id: &Bytes32) {
        if !self
            .service_id_to_user_to_whitelist_status
            .contains_key(service_id)
        {
            self.service_id_to_user_to_whitelist_status
                .insert(service_id, &LookupMap::new(StorageKeys::UserWhitelist));
        }

        if !self
            .service_id_to_user_to_setter_to_indefinite_whitelist_status
            .contains_key(service_id)
        {
            self.service_id_to_user_to_setter_to_indefinite_whitelist_status
                .insert(
                    service_id,
                    &LookupMap::new(StorageKeys::UserSetterIndefiniteWhitelist),
                );
        }
    }
}

#[near_bindgen]
impl Whitelist for NearWhitelist {
    type Address = Address;
    type U256 = U256;

    fn extend_whitelist_expiration(
        &mut self,
        service_id: &Bytes32,
        user: &Address,
        expiration_timestamp: u64,
    ) {
        let mut user_to_whitelist_status = self
            .service_id_to_user_to_whitelist_status
            .remove(service_id)
            .expect("must contain this service");
        let mut whitelist_status = user_to_whitelist_status
            .remove(&user)
            .expect("must contain this user");

        ensure!(
            expiration_timestamp > whitelist_status.expiration_timestamp,
            Error::DoesNotExtendExpiration
        );

        whitelist_status.expiration_timestamp = expiration_timestamp;

        user_to_whitelist_status.insert(user, &whitelist_status);

        self.service_id_to_user_to_whitelist_status
            .insert(service_id, &user_to_whitelist_status);
    }

    fn set_whitelist_expiration(
        &mut self,
        service_id: &Bytes32,
        user: &Address,
        expiration_timestamp: u64,
    ) {
        self.ensure_service_exist(service_id);
        let mut user_to_whitelist_status = self
            .service_id_to_user_to_whitelist_status
            .remove(service_id)
            .expect("must have the service");

        // user has an existing whitelist expiration
        if let Some(mut whitelist_status) = user_to_whitelist_status.remove(&user) {
            whitelist_status.expiration_timestamp = expiration_timestamp;
            user_to_whitelist_status.insert(&user, &whitelist_status);
        } else {
            // insert a new entry for non existing user
            user_to_whitelist_status.insert(
                &user,
                &WhitelistStatus {
                    expiration_timestamp,
                    ..Default::default()
                },
            );
        }
        self.service_id_to_user_to_whitelist_status
            .insert(&service_id, &user_to_whitelist_status);
    }

    fn set_indefinite_whitelist_status(
        &mut self,
        service_id: &Bytes32,
        user: &Address,
        status: bool,
    ) -> U256 {
        // let mut indefinite_whitelist_count = U256::from(0);

        // let mut user_to_whitelist_status = self
        //     .service_id_to_user_to_whitelist_status
        //     .remove(service_id)
        //     .expect("must have a service id");

        // self.service_id_to_user_to_whitelist_status
        //     .insert(service_id, &user_to_whitelist_status);

        // indefinite_whitelist_count
        // TODO: need to know more abount msg.sender and it's equivalent for NEAR
        todo!();
    }

    /// @notice Revokes the indefinite whitelist status granted to the user for
    /// the service by a specific account
    /// @param serviceId Service ID
    /// @param user User address
    /// @param setter Setter of the indefinite whitelist status
    fn revoke_indefinite_whitelist_status(
        &mut self,
        service_id: &Bytes32,
        user: &Address,
        setter: &Address,
    ) -> (bool, U256) {
        self.ensure_service_exist(service_id);

        let mut user_to_whitelist_status = self
            .service_id_to_user_to_whitelist_status
            .remove(service_id)
            .expect("must have a service");

        let mut indefinite_whitelist_count = U256::from(0);

        if let Some(mut whitelist_status) = user_to_whitelist_status.remove(user) {
            indefinite_whitelist_count = whitelist_status.indefinite_whitelist_count;
            indefinite_whitelist_count -= U256::from(1);
            user_to_whitelist_status.insert(user, &whitelist_status);
        }

        let mut user_to_setter_indefinite_whitelist_status = self
            .service_id_to_user_to_setter_to_indefinite_whitelist_status
            .remove(service_id)
            .expect("must have a service id");

        let mut revoked = false;
        if let Some(mut setter_indefinite_whitelist_status) =
            user_to_setter_indefinite_whitelist_status.remove(user)
        {
            setter_indefinite_whitelist_status.insert(setter, &false);
            user_to_setter_indefinite_whitelist_status
                .insert(user, &setter_indefinite_whitelist_status);

            revoked = true;
        }

        self.service_id_to_user_to_whitelist_status
            .insert(service_id, &user_to_whitelist_status);

        (revoked, indefinite_whitelist_count)
    }

    /// @notice Returns if the user is whitelised to use the service
    /// @param serviceId Service ID
    /// @param user User address
    /// @return isWhitelisted If the user is whitelisted
    fn user_is_whitelisted(&self, service_id: &Bytes32, user: &Address) -> bool {
        if let Some(user_to_whitelist_status) =
            self.service_id_to_user_to_whitelist_status.get(service_id)
        {
            if let Some(whitelist_status) = user_to_whitelist_status.get(user) {
                whitelist_status.indefinite_whitelist_count > U256::from(0)
                    || whitelist_status.expiration_timestamp > env::block_timestamp()
            } else {
                false
            }
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::json_types::ValidAccountId;
    use near_sdk::serde::export::TryFrom;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;
    use near_sdk::MockedBlockchain;

    // simple helper function to take a string literal and return a ValidAccountId
    fn to_valid_account(account: &str) -> ValidAccountId {
        ValidAccountId::try_from(account.to_string()).expect("Invalid account")
    }

    // part of writing unit tests is setting up a mock context
    // provide a `predecessor` here, it'll modify the default context
    fn get_context(predecessor: ValidAccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder
    }

    // mark individual unit tests with #[test] for them to be registered and fired
    #[test]
    fn test_no_whitelist() {
        // set up the mock context into the testing environment
        let context = get_context(to_valid_account("foo.near"));
        testing_env!(context.build());
        let whitelist = NearWhitelist::default();
        let user = Address::from("test.near");
        let service = [0; 32];
        assert!(!whitelist.user_is_whitelisted(&service, &user));
    }

    #[test]
    fn test_whitelist_user() {
        // set up the mock context into the testing environment
        let context = get_context(to_valid_account("foo.near"));
        testing_env!(context.build());
        let mut whitelist = NearWhitelist::default();

        let user = Address::from("test.near");
        let service = [0; 32];

        assert!(!whitelist.user_is_whitelisted(&service, &user));

        // whitelist the user
        whitelist.set_whitelist_expiration(&service, &user, 10_000);
        assert!(whitelist.user_is_whitelisted(&service, &user));
    }

    #[test]
    fn serialize_contract() {
        // set up the mock context into the testing environment
        let context = get_context(to_valid_account("foo.near"));
        testing_env!(context.build());
        let mut whitelist = NearWhitelist::default();

        let user = Address::from("test.near");
        let service = [0; 32];

        // whitelisted user
        whitelist.set_whitelist_expiration(&service, &user, 10_000);

        let mut buffer: Vec<u8> = vec![];
        whitelist.serialize(&mut buffer).unwrap();
        dbg!(&buffer);

        let whitelist1 = NearWhitelist::try_from_slice(&mut buffer).unwrap();
        // NOT whitelisted user
        let user1 = Address::from("test1.near");
        assert!(whitelist1.user_is_whitelisted(&service, &user));

        assert!(!whitelist1.user_is_whitelisted(&service, &user1));
    }
}
