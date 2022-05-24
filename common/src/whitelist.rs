#![allow(unused)]
use crate::abi::{Token, U256};
use crate::Error;
use crate::{ensure, keccak_packed, AccessControlRegistryAdminnedWithManager, RoleDeriver};
use crate::{Bytes32, Zero};

/// Trait that implements temporary and permanent whitelists for
/// multiple services identified with a hash
///
/// This trait implements two kinds of whitelisting:
///   (1) Temporary, ends when the expiration timestamp is in the past
///   (2) Indefinite, ends when the indefinite whitelist count is zero
/// Multiple senders can indefinitely whitelist/unwhitelist independently. The
/// user will be considered whitelisted as long as there is at least one active
/// indefinite whitelisting.
///
/// The interface of this contract is not implemented. It should be
/// inherited and its functions should be exposed with a sort of an
/// authorization scheme.
pub trait Whitelist {
    /// The address type for the chain
    type Address: AsRef<[u8]> + Zero;

    /// Returns if the user is whitelised to use the service
    ///
    /// # Argument
    ///
    /// * `service_id` Service ID
    /// * `user` User address
    fn user_is_whitelisted(&self, service_id: &Bytes32, user: &Self::Address) -> bool;

    /// Extends the expiration of the temporary whitelist of the user
    /// for the service
    ///
    /// # Argument
    ///
    /// * `service_id` Service ID
    /// * `user` User address
    /// * `expiration_timestamp` Timestamp at which the temporary whitelist will expire
    fn extend_whitelist_expiration(
        &mut self,
        service_id: &Bytes32,
        user: &Self::Address,
        expiration_timestamp: u64,
    );

    /// Sets the expiration of the temporary whitelist of `user` to be
    /// able to use the service with `serviceId` if the sender has the
    /// whitelist expiration setter role
    ///
    /// # Argument
    ///
    /// * `service_id` Service ID
    /// * `user` User address
    /// * `expiration_timestamp` Timestamp at which the temporary whitelist will expire
    fn set_whitelist_expiration(
        &mut self,
        service_id: &Bytes32,
        user: &Self::Address,
        expiration_timestamp: u64,
    );

    /// Sets the indefinite whitelist status of `user` to be able to
    /// use the service with `serviceId` if the sender has the indefinite
    /// whitelister role
    ///
    /// # Argument
    ///
    /// `service_id` Service ID
    /// `user` User address
    /// `status` Indefinite whitelist status
    fn set_indefinite_whitelist_status(
        &mut self,
        service_id: &Bytes32,
        user: &Self::Address,
        status: bool,
    ) -> U256;

    /// Revokes the indefinite whitelist status granted to the user for
    /// the service by a specific account
    ///
    /// # Argument
    ///
    /// `service_id` Service ID
    /// `user` User address
    /// `setter` Setter of the indefinite whitelist status
    fn revoke_indefinite_whitelist_status(
        &mut self,
        service_id: &Bytes32,
        user: &Self::Address,
        setter: &Self::Address,
    ) -> (bool, U256);
}

pub trait WhitelistRoles {
    fn whitelist_expiration_extender_role_description() -> String {
        String::from("Whitelist expiration extender")
    }

    fn whitelist_expiration_setter_role_description() -> String {
        String::from("Whitelist expiration setter")
    }

    fn indefinite_whitelister_role_description() -> String {
        String::from("Indefinite whitelister")
    }
}

pub trait WhitelistRolesWithManager:
    WhitelistRoles + AccessControlRegistryAdminnedWithManager
{
    /// Returns if the account has the whitelist expiration extender role
    /// or is the manager
    ///
    /// # Arguments
    ///
    /// * `account` Account address
    fn has_whitelist_expiration_extender_role_or_is_manager(&self, account: &Self::Address) -> bool;

    /// Returns if the account has the indefinite whitelister role or is the
    /// manager
    ///
    /// # Arguments
    ///
    /// * `account` Account address
    fn has_indefinite_whitelister_role_or_is_manager(&self, account: &Self::Address) -> bool;

    /// Returns if the account has the whitelist expriation setter role or
    /// is the manager
    ///
    /// # Arguments
    ///
    /// * `account` Account address
    fn has_whitelist_expiration_setter_role_or_is_manager(&self, account: &Self::Address) -> bool;

    fn whitelist_expiration_extender_role(&self) -> Bytes32 {
        RoleDeriver::derive_role(
            self.admin_role(),
            Self::whitelist_expiration_extender_role_description(),
        )
    }

    fn whitelist_expiration_setter_role(&self) -> Bytes32 {
        RoleDeriver::derive_role(
            self.admin_role(),
            Self::whitelist_expiration_setter_role_description(),
        )
    }

    fn indefinite_whitelister_role(&self) -> Bytes32 {
        RoleDeriver::derive_role(
            self.admin_role(),
            Self::indefinite_whitelister_role_description(),
        )
    }
}

/// Whitelist contract that is controlled by a manager
pub trait WhitelistWithManager: Whitelist + WhitelistRolesWithManager {
    /// Extends the expiration of the temporary whitelist of `user` to
    /// be able to use the service with `service_id` if the sender has the
    /// whitelist expiration extender role
    ///
    /// # Arguments
    ///
    /// * `service_id` Service ID
    /// * `user` User address
    /// * `expiration_timestamp` Timestamp at which the temporary whitelist will expire
    fn extend_whitelist_expiration(
        &mut self,
        service_id: &Bytes32,
        user: &<Self as Whitelist>::Address,
        expiration_timestamp: u64,
    );

    /// Sets the expiration of the temporary whitelist of `user` to be
    /// able to use the service with `service_id` if the sender has the
    /// whitelist expiration setter role
    ///
    /// # Arguments
    ///
    /// * `service_id` Service ID
    /// * `user` User address
    /// * `expiration_timestamp` Timestamp at which the temporary whitelist will expire
    fn set_whitelist_expiration(
        &mut self,
        service_id: &Bytes32,
        user: &<Self as Whitelist>::Address,
        expiration_timestamp: u64,
    );

    /// Sets the indefinite whitelist status of `user` to be able to
    /// use the service with `service_id` if the sender has the indefinite whitelister role
    ///
    /// # Arguments
    ///
    /// * `service_id` Service ID
    /// * `user` User address
    /// * `status` Indefinite whitelist status
    fn set_indefinite_whitelist_status(
        &mut self,
        service_id: &Bytes32,
        user: &<Self as Whitelist>::Address,
        status: bool,
    ) -> U256;

    /// Revokes the indefinite whitelist status granted to the user for
    /// the service by a specific account
    ///
    /// # Arguments
    ///
    /// * `service_id` Service ID
    /// * `user` User address
    /// * `setter` Setter address
    fn revoke_indefinite_whitelist_status(
        &mut self,
        service_id: &Bytes32,
        user: &<Self as Whitelist>::Address,
        setter: &<Self as Whitelist>::Address,
    ) -> (bool, U256);
}
