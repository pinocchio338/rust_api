use crate::utils::ReadWrite;
use crate::{msg_sender, Address};
use api3_common::abi::{Token, U256};
use api3_common::{
    ensure, keccak_packed, AccessControlRegistry, AccessControlRegistryAdminnedWithManager,
    Bytes32, Error, Whitelist, WhitelistRoles, WhitelistRolesWithManager, WhitelistWithManager,
    Zero, BYTES32_ZERO,
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use std::ops::{Add, Sub};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct WhitelistStatus {
    expiration_timestamp: u64,
    /// originally uint192, that is u128 and u64 combined
    indefinite_whitelist_count: Bytes32,
}

impl Default for WhitelistStatus {
    fn default() -> Self {
        Self {
            expiration_timestamp: 0,
            indefinite_whitelist_count: BYTES32_ZERO,
        }
    }
}

pub struct NearWhitelist<'a, Access: AccessControlRegistry<Address = Address>> {
    access: &'a Access,
    service_id_to_user_to_whitelist_status: ReadWrite<'a, LookupMap<Bytes32, WhitelistStatus>>,
    service_id_to_user_to_setter_to_indefinite_whitelist_status:
        ReadWrite<'a, LookupMap<Bytes32, bool>>,
}

impl<'a, Access: AccessControlRegistry<Address = Address>> NearWhitelist<'a, Access> {
    pub fn requires_write(
        access: &'a Access,
        service_id_to_user_to_whitelist_status: &'a mut LookupMap<Bytes32, WhitelistStatus>,
        service_id_to_user_to_setter_to_indefinite_whitelist_status: &'a mut LookupMap<
            Bytes32,
            bool,
        >,
    ) -> Self {
        Self {
            access,
            service_id_to_user_to_whitelist_status: ReadWrite::Write(
                service_id_to_user_to_whitelist_status,
            ),
            service_id_to_user_to_setter_to_indefinite_whitelist_status: ReadWrite::Write(
                service_id_to_user_to_setter_to_indefinite_whitelist_status,
            ),
        }
    }

    pub fn read_only(
        access: &'a Access,
        service_id_to_user_to_whitelist_status: &'a LookupMap<Bytes32, WhitelistStatus>,
        service_id_to_user_to_setter_to_indefinite_whitelist_status: &'a LookupMap<Bytes32, bool>,
    ) -> Self {
        Self {
            access,
            service_id_to_user_to_whitelist_status: ReadWrite::ReadOnly(
                service_id_to_user_to_whitelist_status,
            ),
            service_id_to_user_to_setter_to_indefinite_whitelist_status: ReadWrite::ReadOnly(
                service_id_to_user_to_setter_to_indefinite_whitelist_status,
            ),
        }
    }

    pub fn data_feed_id_to_reader_to_setter_to_indefinite_whitelist_status(
        &self,
        data_feed_id: &Bytes32,
        reader: &Bytes32,
        setter: &Bytes32,
    ) -> Option<bool> {
        let key = Self::triple_hash(&data_feed_id, reader, setter);
        let indefinite = match &self.service_id_to_user_to_setter_to_indefinite_whitelist_status {
            ReadWrite::ReadOnly(m) => *m,
            ReadWrite::Write(m) => *m,
        };
        indefinite.get(&key)
    }

    pub fn data_feed_id_to_whitelist_status(
        &self,
        data_feed_id: &Bytes32,
        reader: &Bytes32,
    ) -> Option<(u64, Bytes32)> {
        let key = Self::double_hash(&data_feed_id, reader);
        let s = match &self.service_id_to_user_to_whitelist_status {
            ReadWrite::ReadOnly(s) => *s,
            ReadWrite::Write(s) => *s,
        };
        s.get(&key)
            .map(|w| (w.expiration_timestamp, w.indefinite_whitelist_count))
    }

    fn double_hash(service_id: &Bytes32, address: &[u8]) -> Bytes32 {
        keccak_packed(&[
            Token::FixedBytes(service_id.to_vec()),
            Token::FixedBytes(address.to_vec()),
        ])
    }

    fn triple_hash(service_id: &Bytes32, address0: &[u8], address1: &[u8]) -> Bytes32 {
        keccak_packed(&[
            Token::FixedBytes(service_id.to_vec()),
            Token::FixedBytes(address0.to_vec()),
            Token::FixedBytes(address1.to_vec()),
        ])
    }
}

impl<'a, Access: AccessControlRegistry<Address = Address>> Whitelist for NearWhitelist<'a, Access> {
    type Address = Address;

    /// Returns if the user is whitelisted to use the service
    /// `service_id` Service ID
    /// `user` User address
    fn user_is_whitelisted(&self, service_id: &Bytes32, user: &Address) -> bool {
        let hash = Self::double_hash(service_id, &user.0);
        let s = match &self.service_id_to_user_to_whitelist_status {
            ReadWrite::ReadOnly(s) => *s,
            ReadWrite::Write(s) => *s,
        };
        s.get(&hash)
            .map(|status| {
                let count = U256::from_big_endian(&status.indefinite_whitelist_count);
                count > U256::from(0)
                    || status.expiration_timestamp > near_sdk::env::block_timestamp()
            })
            .unwrap_or(false)
    }

    fn extend_whitelist_expiration(
        &mut self,
        service_id: &Bytes32,
        user: &Self::Address,
        expiration_timestamp: u64,
    ) {
        let m = match &mut self.service_id_to_user_to_whitelist_status {
            ReadWrite::ReadOnly(_) => panic!("wrong privilege"),
            ReadWrite::Write(m) => m,
        };

        let hash = Self::double_hash(service_id, &user.0);

        let mut whitelist_status = (*m).remove(&hash).expect("must contain this service");
        ensure!(
            expiration_timestamp > whitelist_status.expiration_timestamp,
            Error::DoesNotExtendExpiration
        )
        .unwrap();

        whitelist_status.expiration_timestamp = expiration_timestamp;
        (*m).insert(&hash, &whitelist_status);
    }

    fn set_whitelist_expiration(
        &mut self,
        service_id: &Bytes32,
        user: &Self::Address,
        expiration_timestamp: u64,
    ) {
        let m = match &mut self.service_id_to_user_to_whitelist_status {
            ReadWrite::ReadOnly(_) => panic!("wrong privilege"),
            ReadWrite::Write(m) => m,
        };
        let hash = Self::double_hash(service_id, &user.0);
        let mut whitelist_status = (*m).remove(&hash).expect("must contain this service");
        whitelist_status.expiration_timestamp = expiration_timestamp;
        (*m).insert(&hash, &whitelist_status);
    }

    fn set_indefinite_whitelist_status(
        &mut self,
        service_id: &Bytes32,
        user: &Self::Address,
        status: bool,
    ) -> U256 {
        let w_hash = Self::double_hash(service_id, &user.0);
        let i_hash = Self::triple_hash(service_id, &user.0, &msg_sender().0);

        let whitelist = match &mut self.service_id_to_user_to_whitelist_status {
            ReadWrite::ReadOnly(_) => panic!("wrong privilege"),
            ReadWrite::Write(m) => m,
        };
        let indefinite = match &mut self.service_id_to_user_to_setter_to_indefinite_whitelist_status
        {
            ReadWrite::ReadOnly(_) => panic!("wrong privilege"),
            ReadWrite::Write(m) => m,
        };

        let mut indefinite_count = whitelist
            .get(&w_hash)
            .map(|f| U256::from_big_endian(&f.indefinite_whitelist_count))
            .unwrap_or_else(|| U256::from(0u32));
        let indefinite_status = indefinite.get(&i_hash).unwrap_or(false);

        if status && !indefinite_status {
            indefinite.remove(&i_hash);
            indefinite.insert(&i_hash, &true);
            let mut whitelist_status = whitelist.remove(&w_hash).unwrap_or_default();
            indefinite_count = indefinite_count.add(U256::from(1u8));
            whitelist_status.indefinite_whitelist_count = Bytes32::from(&indefinite_count);
            whitelist.insert(&w_hash, &whitelist_status);
        } else if !status && indefinite_status {
            indefinite.remove(&i_hash);
            indefinite.insert(&i_hash, &false);
            let mut whitelist_status = whitelist.remove(&w_hash).unwrap_or_default();
            indefinite_count = indefinite_count.sub(U256::from(1u8));
            whitelist_status.indefinite_whitelist_count = Bytes32::from(&indefinite_count);
            whitelist.insert(&w_hash, &whitelist_status);
        }

        indefinite_count
    }

    fn revoke_indefinite_whitelist_status(
        &mut self,
        service_id: &Bytes32,
        user: &Self::Address,
        setter: &Self::Address,
    ) -> (bool, U256) {
        let user_hash = Self::double_hash(service_id, &user.0);
        let setter_hash = Self::triple_hash(service_id, &user.0, &setter.0);

        let indefinite = match &mut self.service_id_to_user_to_setter_to_indefinite_whitelist_status
        {
            ReadWrite::ReadOnly(_) => panic!("wrong privilege"),
            ReadWrite::Write(m) => m,
        };

        let whitelist = match &mut self.service_id_to_user_to_whitelist_status {
            ReadWrite::ReadOnly(_) => panic!("wrong privilege"),
            ReadWrite::Write(m) => m,
        };

        let mut indefinite_count = whitelist
            .get(&user_hash)
            .map(|f| U256::from_big_endian(&f.indefinite_whitelist_count))
            .unwrap_or_else(|| U256::from(0u32));

        let indefinite_status = indefinite.get(&setter_hash).unwrap_or(false);
        if indefinite_status {
            indefinite.remove(&setter_hash);
            indefinite.insert(&setter_hash, &false);

            let mut whitelist_status = whitelist.remove(&user_hash).unwrap_or_default();
            indefinite_count = indefinite_count.sub(U256::from(1u8));
            whitelist_status.indefinite_whitelist_count = Bytes32::from(&indefinite_count);
            whitelist.insert(&user_hash, &whitelist_status);
            (true, indefinite_count)
        } else {
            (false, indefinite_count)
        }
    }
}

impl<'a, Access: AccessControlRegistry<Address = Address>> WhitelistRolesWithManager
    for NearWhitelist<'a, Access>
{
    fn has_whitelist_expiration_extender_role_or_is_manager(
        &self,
        account: &Self::Address,
    ) -> bool {
        self.manager() == account
            || self
                .access
                .has_role(&self.whitelist_expiration_extender_role(), account)
    }

    fn has_indefinite_whitelister_role_or_is_manager(&self, account: &Self::Address) -> bool {
        self.manager() == account
            || self
                .access
                .has_role(&self.indefinite_whitelister_role(), account)
    }

    fn has_whitelist_expiration_setter_role_or_is_manager(&self, account: &Self::Address) -> bool {
        self.manager() == account
            || self
                .access
                .has_role(&self.whitelist_expiration_setter_role(), account)
    }
}

impl<'a, Access: AccessControlRegistry<Address = Address>> WhitelistRoles
    for NearWhitelist<'a, Access>
{
}

impl<'a, Access: AccessControlRegistry<Address = Address>> AccessControlRegistryAdminnedWithManager
    for NearWhitelist<'a, Access>
{
    type Address = Address;

    fn manager(&self) -> &Self::Address {
        self.access.manager()
    }

    fn admin_role_description(&self) -> String {
        self.access.admin_role_description()
    }

    fn admin_role_description_hash(&self) -> Bytes32 {
        self.access.admin_role_description_hash()
    }

    fn admin_role(&self) -> Bytes32 {
        self.access.admin_role()
    }
}

impl<'a, Access: AccessControlRegistry<Address = Address>> WhitelistWithManager
    for NearWhitelist<'a, Access>
{
    fn extend_whitelist_expiration(
        &mut self,
        service_id: &Bytes32,
        user: &<Self as Whitelist>::Address,
        expiration_timestamp: u64,
    ) {
        ensure!(
            self.has_whitelist_expiration_setter_role_or_is_manager(&msg_sender()),
            Error::DoesNotExtendExpiration
        )
        .unwrap();
        ensure!(*service_id != [0; 32], Error::ServiceIdZero).unwrap();
        ensure!(*user.as_ref() != [0; 32], Error::UserAddressZero).unwrap();
        Whitelist::extend_whitelist_expiration(self, service_id, user, expiration_timestamp);
    }

    fn set_whitelist_expiration(
        &mut self,
        service_id: &Bytes32,
        user: &<Self as Whitelist>::Address,
        expiration_timestamp: u64,
    ) {
        ensure!(
            self.has_whitelist_expiration_setter_role_or_is_manager(&msg_sender()),
            Error::DoesNotExtendExpiration
        )
        .unwrap();

        ensure!(!service_id.is_zero(), Error::ServiceIdZero).unwrap();
        ensure!(!user.0.is_zero(), Error::UserAddressZero).unwrap();

        Whitelist::set_whitelist_expiration(self, service_id, user, expiration_timestamp)
    }

    fn set_indefinite_whitelist_status(
        &mut self,
        service_id: &Bytes32,
        user: &<Self as Whitelist>::Address,
        status: bool,
    ) -> U256 {
        ensure!(
            self.has_indefinite_whitelister_role_or_is_manager(&msg_sender()),
            Error::CannotSetIndefiniteStatus
        )
        .unwrap();

        ensure!(!service_id.is_zero(), Error::ServiceIdZero).unwrap();
        ensure!(!user.0.is_zero(), Error::UserAddressZero).unwrap();

        Whitelist::set_indefinite_whitelist_status(self, service_id, user, status)
    }

    fn revoke_indefinite_whitelist_status(
        &mut self,
        service_id: &Bytes32,
        user: &<Self as Whitelist>::Address,
        setter: &<Self as Whitelist>::Address,
    ) -> (bool, U256) {
        ensure!(
            self.has_indefinite_whitelister_role_or_is_manager(setter),
            Error::CannotSetIndefiniteStatus
        )
        .unwrap();
        Whitelist::revoke_indefinite_whitelist_status(self, service_id, user, setter)
    }
}
