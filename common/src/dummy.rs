//! A collection of default implementations

use crate::abi::U256;
use crate::{
    AccessControlRegistry, AccessControlRegistryAdminnedWithManager, Bytes32, Error, Whitelist,
    WhitelistRoles, WhitelistRolesWithManager, WhitelistWithManager, Zero,
};

pub struct DummyWhitelist<Address: AsRef<[u8]> + Zero + Default + PartialEq> {
    manager: Address,
}

impl<Address: AsRef<[u8]> + Zero + Default + PartialEq> Default for DummyWhitelist<Address> {
    fn default() -> Self {
        Self {
            manager: Address::default(),
        }
    }
}

impl<Address> Whitelist for DummyWhitelist<Address>
where
    Address: AsRef<[u8]> + Zero + Default + PartialEq,
{
    type Address = Address;
    fn user_is_whitelisted(&self, _service_id: &Bytes32, _user: &Self::Address) -> bool {
        false
    }
    fn extend_whitelist_expiration(
        &mut self,
        _service_id: &Bytes32,
        _user: &Self::Address,
        _expiration_timestamp: u64,
    ) {
    }

    fn set_whitelist_expiration(
        &mut self,
        _service_id: &Bytes32,
        _user: &Self::Address,
        _expiration_timestamp: u64,
    ) {
    }

    fn set_indefinite_whitelist_status(
        &mut self,
        _service_id: &Bytes32,
        _user: &Self::Address,
        _status: bool,
    ) -> U256 {
        U256::from(0u8)
    }

    fn revoke_indefinite_whitelist_status(
        &mut self,
        _service_id: &Bytes32,
        _user: &Self::Address,
        _setter: &Self::Address,
    ) -> (bool, U256) {
        (true, U256::from(0u8))
    }
}

impl<Address> WhitelistRoles for DummyWhitelist<Address> where
    Address: AsRef<[u8]> + Default + PartialEq + Zero
{
}

impl<Address> WhitelistRolesWithManager for DummyWhitelist<Address>
where
    Address: AsRef<[u8]> + Default + PartialEq + Zero,
{
    fn has_whitelist_expiration_extender_role_or_is_manager(
        &self,
        _account: &Self::Address,
    ) -> bool {
        true
    }

    fn has_indefinite_whitelister_role_or_is_manager(&self, _account: &Self::Address) -> bool {
        true
    }

    fn has_whitelist_expiration_setter_role_or_is_manager(&self, _account: &Self::Address) -> bool {
        true
    }
}

impl<Address> AccessControlRegistryAdminnedWithManager for DummyWhitelist<Address>
where
    Address: AsRef<[u8]> + Default + PartialEq + Zero,
{
    type Address = Address;

    fn manager(&self) -> &Self::Address {
        &self.manager
    }

    fn admin_role_description(&self) -> String {
        String::from("")
    }

    fn admin_role_description_hash(&self) -> Bytes32 {
        Bytes32::default()
    }

    fn admin_role(&self) -> Bytes32 {
        Bytes32::default()
    }
}

impl<Address> WhitelistWithManager for DummyWhitelist<Address>
where
    Address: AsRef<[u8]> + Zero + Default + PartialEq,
{
    fn extend_whitelist_expiration(
        &mut self,
        _service_id: &Bytes32,
        _user: &<Self as Whitelist>::Address,
        _expiration_timestamp: u64,
    ) {
    }

    fn set_whitelist_expiration(&mut self, _service_id: &Bytes32, _user: &<Self as Whitelist>::Address, _expiration_timestamp: u64) {
    }

    fn set_indefinite_whitelist_status(&mut self, _service_id: &Bytes32, _user: &<Self as Whitelist>::Address, _status: bool) -> U256 {
        U256::from(0u8)
    }

    fn revoke_indefinite_whitelist_status(&mut self, _service_id: &Bytes32, _user: &<Self as Whitelist>::Address, _setter: &<Self as Whitelist>::Address) -> (bool, U256) {
        (false, U256::from(0u8))
    }
}

pub struct DummyAccess<Address: AsRef<[u8]> + Zero + Default + PartialEq> {
    manager: Address,
}

impl<Address: AsRef<[u8]> + Zero + Default + PartialEq> Default for DummyAccess<Address> {
    fn default() -> Self {
        Self {
            manager: Address::default(),
        }
    }
}

impl<Address> AccessControlRegistryAdminnedWithManager for DummyAccess<Address>
where
    Address: AsRef<[u8]> + Zero + Default + PartialEq,
{
    type Address = Address;
    fn manager(&self) -> &Self::Address {
        &self.manager
    }
    fn admin_role_description(&self) -> String {
        String::from("")
    }
    fn admin_role_description_hash(&self) -> Bytes32 {
        Bytes32::default()
    }
    fn admin_role(&self) -> Bytes32 {
        Bytes32::default()
    }
}

impl<Address> AccessControlRegistry for DummyAccess<Address>
where
    Address: AsRef<[u8]> + Zero + Default + PartialEq,
{
    fn has_role(&self, _role: &Bytes32, _who: &Self::Address) -> bool {
        true
    }
    fn grant_role(&mut self, _role: &Bytes32, _who: &Self::Address) -> Result<(), Error> {
        Ok(())
    }
    fn get_role_admin(&self, _role: &Bytes32) -> Option<Bytes32> {
        Some(Bytes32::default())
    }
    fn set_role_admin(&mut self, _role: &Bytes32, _role_admin: Bytes32) -> Result<(), Error> {
        Ok(())
    }
    fn renounce_role(&mut self, _role: &Bytes32, _account: &Self::Address) -> Result<(), Error> {
        Ok(())
    }

    fn revoke_role(&mut self, _role: &Bytes32, _account: &Self::Address) -> Result<(), Error> {
        Ok(())
    }
}
