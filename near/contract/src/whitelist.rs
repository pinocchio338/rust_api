/// derived from Whitelist.sol
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

use crate::types::Address;
use crate::types::U256;

near_sdk::setup_alloc!();

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys {
    Service,
    UserWhitelist,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Default)]
struct WhitelistStatus {
    expiration_timestamp: u64,
    /// orignally uint192, that is u128 and u64 combined
    indefinite_whitelist_count: U256,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
struct Contract {
    service_id_to_user_to_whitelist_status: LookupMap<String, LookupMap<Address, WhitelistStatus>>,
}

#[near_bindgen]
impl Contract {
    pub fn new() -> Self {
        Contract {
            service_id_to_user_to_whitelist_status: LookupMap::new(StorageKeys::Service),
        }
    }

    /// @notice Returns if the user is whitelised to use the service
    /// @param serviceId Service ID
    /// @param user User address
    /// @return isWhitelisted If the user is whitelisted
    pub fn user_is_whitelisted(&self, service_id: &String, user: &Address) -> bool {
        if let Some(user_to_whitelist_status) =
            self.service_id_to_user_to_whitelist_status.get(service_id)
        {
            user_to_whitelist_status.contains_key(user)
        } else {
            false
        }
    }

    pub fn whitelist_user(&mut self, service_id: &String, user: &Address) {
        if let Some(mut user_to_whitelist_status) = self
            .service_id_to_user_to_whitelist_status
            .remove(service_id)
        {
            user_to_whitelist_status.insert(user, &WhitelistStatus::default());

            self.service_id_to_user_to_whitelist_status
                .insert(service_id, &user_to_whitelist_status);
        } else {
            println!("no such service..: {}", service_id);
            self.add_service(service_id);
            self.whitelist_user(service_id, user);
        }
    }

    pub fn add_service(&mut self, service_id: &String) {
        assert!(
            !self
                .service_id_to_user_to_whitelist_status
                .contains_key(service_id),
            "must not have an existing service_id"
        );

        self.service_id_to_user_to_whitelist_status
            .insert(service_id, &LookupMap::new(StorageKeys::UserWhitelist));
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
        let contract = Contract::new();
        let user = Address::from([0; 20]);
        let service = "service1".to_string();
        assert!(!contract.user_is_whitelisted(&service, &user));
    }

    #[test]
    fn test_whitelist_user() {
        // set up the mock context into the testing environment
        let context = get_context(to_valid_account("foo.near"));
        testing_env!(context.build());
        let mut contract = Contract::new();

        let user = Address::from([0; 20]);
        let service = "service1".to_string();

        assert!(!contract.user_is_whitelisted(&service, &user));

        // whitelist the user
        contract.whitelist_user(&service, &user);
        assert!(contract.user_is_whitelisted(&service, &user));
    }

    #[test]
    fn serialize_contract() {
        // set up the mock context into the testing environment
        let context = get_context(to_valid_account("foo.near"));
        testing_env!(context.build());
        let mut contract = Contract::new();

        let user = Address::from([0; 20]);
        let service = "service1".to_string();

        // whitelisted user
        contract.whitelist_user(&service, &user);

        let mut buffer: Vec<u8> = vec![];
        contract.serialize(&mut buffer).unwrap();
        dbg!(&buffer);

        let contract1 = Contract::try_from_slice(&mut buffer).unwrap();
        // NOT whitelisted user
        let user1 = Address::from([1; 20]);
        assert!(contract1.user_is_whitelisted(&service, &user));

        assert!(!contract1.user_is_whitelisted(&service, &user1));
    }

    #[test]
    #[should_panic]
    fn must_error_when_adding_same_service_multiple_times() {
        let context = get_context(to_valid_account("foo.near"));
        testing_env!(context.build());
        let mut contract = Contract::new();

        let service = "service1".to_string();

        contract.add_service(&service);
        contract.add_service(&service);
    }
}
