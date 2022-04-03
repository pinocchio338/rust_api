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
use near_sdk::{env, near_bindgen};
use std::io;

use crate::types::Address;
use crate::types::U256;

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Default)]
struct WhitelistStatus {
    expiration_timestamp: u64,
    /// orignally uint192, that is u128 and u64 combined
    ///
    indefinite_whitelist_count: U256,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
struct Contract {
    user_to_whitelist_status: LookupMap<Address, WhitelistStatus>,
}

#[near_bindgen]
impl Contract {
    pub fn new() -> Self {
        Contract {
            user_to_whitelist_status: LookupMap::new(b'u'),
        }
    }

    /// @notice Returns if the user is whitelised to use the service
    /// @param serviceId Service ID
    /// @param user User address
    /// @return isWhitelisted If the user is whitelisted
    pub fn user_is_whitelisted(&self, user: &Address) -> bool {
        self.user_to_whitelist_status.contains_key(user)
    }

    pub fn whitelist_user(&mut self, user: &Address) {
        self.user_to_whitelist_status
            .insert(user, &WhitelistStatus::default());
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
        assert!(!contract.user_is_whitelisted(&user));
    }

    #[test]
    fn test_whitelist_user() {
        // set up the mock context into the testing environment
        let context = get_context(to_valid_account("foo.near"));
        testing_env!(context.build());
        let mut contract = Contract::new();
        let user = Address::from([0; 20]);
        assert!(!contract.user_is_whitelisted(&user));

        // whitelist the user
        contract.whitelist_user(&user);
        assert!(contract.user_is_whitelisted(&user));
    }
}
