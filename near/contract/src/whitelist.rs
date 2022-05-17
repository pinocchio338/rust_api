use api3_common::Bytes32;

// use near_sdk::collections::LookupMap;
// use api3_common::abi::{Token, U256};
// use api3_common::{AccessControlRegistryAdminnedWithManager, Bytes32, ensure, Error, keccak_packed, Whitelist, WhitelistRoles, WhitelistRolesWithManager, WhitelistWithManager};
// use crate::{Address, msg_sender};
// use crate::utils::ReadWrite;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
// use near_sdk::BorshStorageKey;
//
// /// Near whitelist implementation
// /// Currently a dummy implementation
// #[derive(BorshStorageKey, BorshSerialize)]
// pub enum StorageKeys {
//     Service,
//     UserWhitelist,
//     ServiceWithSetter,
//     UserSetterIndefiniteWhitelist,
// }
//
#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct WhitelistStatus {
    expiration_timestamp: u64,
    /// orignally uint192, that is u128 and u64 combined
    indefinite_whitelist_count: Bytes32,
}
//
// pub struct NearWhitelist<'a, Access: AccessControlRegistryAdminnedWithManager> {
//     access: &'a Access,
//     service_id_to_user_to_whitelist_status: ReadWrite<'a, LookupMap<Bytes32, WhitelistStatus>>,
//     service_id_to_user_to_setter_to_indefinite_whitelist_status: ReadWrite<'a, LookupMap<Bytes32, bool>>,
// }
//
// impl <'a, Access: AccessControlRegistryAdminnedWithManager> NearWhitelist<'a, Access> {
//     pub fn requires_write(
//         access: &'a Access,
//         service_id_to_user_to_whitelist_status: &'a mut LookupMap<Bytes32, WhitelistStatus>,
//         service_id_to_user_to_setter_to_indefinite_whitelist_status: &'a mut LookupMap<Bytes32, bool>,
//     ) -> Self {
//         Self {
//             access,
//             service_id_to_user_to_whitelist_status: ReadWrite::Write(service_id_to_user_to_whitelist_status),
//             service_id_to_user_to_setter_to_indefinite_whitelist_status: ReadWrite::Write(service_id_to_user_to_setter_to_indefinite_whitelist_status)
//         }
//     }
//
//     pub fn read_only(
//         access: &'a Access,
//         service_id_to_user_to_whitelist_status: &'a LookupMap<Bytes32, WhitelistStatus>,
//         service_id_to_user_to_setter_to_indefinite_whitelist_status: &'a LookupMap<Bytes32, bool>,
//     ) -> Self {
//         Self {
//             access,
//             service_id_to_user_to_whitelist_status: ReadWrite::ReadOnly(service_id_to_user_to_whitelist_status),
//             service_id_to_user_to_setter_to_indefinite_whitelist_status: ReadWrite::ReadOnly(service_id_to_user_to_setter_to_indefinite_whitelist_status)
//         }
//     }
// }
//
// impl <'a, Access: AccessControlRegistryAdminnedWithManager> NearWhitelist<'a, Access> {
//     // /// ensure that the service_id is in the lookup map
//     // fn ensure_service_exist(&mut self, service_id: &Bytes32) {
//     //     if !self
//     //         .service_id_to_user_to_whitelist_status
//     //         .contains_key(service_id)
//     //     {
//     //         self.service_id_to_user_to_whitelist_status
//     //             .insert(service_id, &LookupMap::new(StorageKeys::UserWhitelist));
//     //     }
//     //
//     //     if !self
//     //         .service_id_to_user_to_setter_to_indefinite_whitelist_status
//     //         .contains_key(service_id)
//     //     {
//     //         self.service_id_to_user_to_setter_to_indefinite_whitelist_status
//     //             .insert(
//     //                 service_id,
//     //                 &LookupMap::new(StorageKeys::UserSetterIndefiniteWhitelist),
//     //             );
//     //     }
//     // }
// }
//
// impl  <'a, Access: AccessControlRegistryAdminnedWithManager> Whitelist for NearWhitelist<'a, Access> {
//     type Address = Address;
//
//     /// Returns if the user is whitelisted to use the service
//     /// `service_id` Service ID
//     /// `user` User address
//     fn user_is_whitelisted(&self, service_id: &Bytes32, user: &Address) -> bool {
//         let hash = keccak_packed(&[
//             Token::FixedBytes(service_id.to_vec()),
//             Token::FixedBytes(user.0.to_vec())
//         ]);
//         let s = match &self.service_id_to_user_to_whitelist_status {
//             ReadWrite::ReadOnly(s) => *s,
//             ReadWrite::Write(s) => *(&*s)
//         };
//         s.get(&hash).map(|status| {
//             let count = U256::from_big_endian(&status.indefinite_whitelist_count);
//             count > U256::from(0) || status.expiration_timestamp > near_sdk::env::block_timestamp()
//         }).unwrap_or(false)
//     }
//
//     fn extend_whitelist_expiration(&mut self, service_id: &Bytes32, user: &Self::Address, expiration_timestamp: u64) {
//         let m = match &mut self.service_id_to_user_to_whitelist_status {
//             ReadWrite::ReadOnly(_) => panic!("wrong privilege"),
//             ReadWrite::Write(m) => m
//         };
//
//         let hash = keccak_packed(&[
//             Token::FixedBytes(service_id.to_vec()),
//             Token::FixedBytes(user.0.as_ref().to_vec())
//         ]);
//
//         let mut whitelist_status = (*m).remove(&hash)
//             .expect("must contain this service");
//         ensure!(
//             expiration_timestamp > whitelist_status.expiration_timestamp,
//             Error::DoesNotExtendExpiration
//         ).unwrap();
//
//         whitelist_status.expiration_timestamp = expiration_timestamp;
//         (*m).insert(&hash, &whitelist_status);
//     }
//
//     fn set_whitelist_expiration(&mut self, _service_id: &Bytes32, _user: &Self::Address, _expiration_timestamp: u64) {
//         todo!()
//     }
//
//     fn set_indefinite_whitelist_status(&mut self, _service_id: &Bytes32, _user: &Self::Address, _status: bool) -> U256 {
//         todo!()
//     }
//
//     fn revoke_indefinite_whitelist_status(&mut self, _service_id: &Bytes32, _user: &Self::Address, _setter: &Self::Address) -> (bool, U256) {
//         todo!()
//     }
// }
//
// impl <'a, Access: AccessControlRegistryAdminnedWithManager<Address = Address>> WhitelistRolesWithManager for NearWhitelist<'a, Access> {
//     fn has_whitelist_expiration_extender_role_or_is_manager(&self, _account: &Self::Address) -> bool {
//         todo!()
//     }
//
//     fn has_indefinite_whitelister_role_or_is_manager(&self, _account: Self::Address) -> bool {
//         todo!()
//     }
//
//     fn has_whitelist_expiration_setter_role_or_is_manager(&self, _account: Self::Address) -> bool {
//         todo!()
//     }
// }
//
// impl  <'a, Access: AccessControlRegistryAdminnedWithManager> WhitelistRoles for NearWhitelist<'a, Access> {}
//
// impl  <'a, Access: AccessControlRegistryAdminnedWithManager> AccessControlRegistryAdminnedWithManager for NearWhitelist<'a, Access> {
//     type Address = <Access as AccessControlRegistryAdminnedWithManager>::Address;
//
//     fn manager(&self) -> &Self::Address {
//         self.access.manager()
//     }
//
//     fn admin_role_description(&self) -> String {
//         self.access.admin_role_description()
//     }
//
//     fn admin_role_description_hash(&self) -> Bytes32 {
//         self.access.admin_role_description_hash()
//     }
//
//     fn admin_role(&self) -> Bytes32 {
//         self.access.admin_role()
//     }
// }
//
// impl <'a, Access: AccessControlRegistryAdminnedWithManager<Address = Address>> WhitelistWithManager for NearWhitelist<'a, Access> {
//     fn extend_whitelist_expiration(&mut self, service_id: &Bytes32, user: &<Self as Whitelist>::Address, expiration_timestamp: u64) {
//         ensure!(
//             self.has_whitelist_expiration_setter_role_or_is_manager(msg_sender()),
//             Error::DoesNotExtendExpiration
//         ).unwrap();
//         ensure!(*service_id != [0; 32], Error::ServiceIdZero).unwrap();
//         ensure!(*user.as_ref() != [0; 32], Error::UserAddressZero).unwrap();
//         Whitelist::extend_whitelist_expiration(self, service_id, user, expiration_timestamp);
//     }
//
//     //
//     // fn set_whitelist_expiration(
//     //     &mut self,
//     //     service_id: &Bytes32,
//     //     user: &Address,
//     //     expiration_timestamp: u64,
//     // ) {
//     //     self.ensure_service_exist(service_id);
//     //     let mut user_to_whitelist_status = self
//     //         .service_id_to_user_to_whitelist_status
//     //         .remove(service_id)
//     //         .expect("must have the service");
//     //
//     //     // user has an existing whitelist expiration
//     //     if let Some(mut whitelist_status) = user_to_whitelist_status.remove(&user) {
//     //         whitelist_status.expiration_timestamp = expiration_timestamp;
//     //         user_to_whitelist_status.insert(&user, &whitelist_status);
//     //     } else {
//     //         // insert a new entry for non existing user
//     //         user_to_whitelist_status.insert(
//     //             &user,
//     //             &WhitelistStatus {
//     //                 expiration_timestamp,
//     //                 ..Default::default()
//     //             },
//     //         );
//     //     }
//     //     self.service_id_to_user_to_whitelist_status
//     //         .insert(&service_id, &user_to_whitelist_status);
//     // }
//     //
//     // fn set_indefinite_whitelist_status(
//     //     &mut self,
//     //     service_id: &Bytes32,
//     //     user: &Address,
//     //     status: bool,
//     // ) -> U256 {
//     //     // let mut indefinite_whitelist_count = U256::from(0);
//     //
//     //     // let mut user_to_whitelist_status = self
//     //     //     .service_id_to_user_to_whitelist_status
//     //     //     .remove(service_id)
//     //     //     .expect("must have a service id");
//     //
//     //     // self.service_id_to_user_to_whitelist_status
//     //     //     .insert(service_id, &user_to_whitelist_status);
//     //
//     //     // indefinite_whitelist_count
//     //     // TODO: need to know more abount msg.sender and it's equivalent for NEAR
//     //     todo!();
//     // }
//     //
//     // /// @notice Revokes the indefinite whitelist status granted to the user for
//     // /// the service by a specific account
//     // /// @param serviceId Service ID
//     // /// @param user User address
//     // /// @param setter Setter of the indefinite whitelist status
//     // fn revoke_indefinite_whitelist_status(
//     //     &mut self,
//     //     service_id: &Bytes32,
//     //     user: &Address,
//     //     setter: &Address,
//     // ) -> (bool, U256) {
//     //     self.ensure_service_exist(service_id);
//     //
//     //     let mut user_to_whitelist_status = self
//     //         .service_id_to_user_to_whitelist_status
//     //         .remove(service_id)
//     //         .expect("must have a service");
//     //
//     //     let mut indefinite_whitelist_count = U256::from(0);
//     //
//     //     if let Some(mut whitelist_status) = user_to_whitelist_status.remove(user) {
//     //         indefinite_whitelist_count = whitelist_status.indefinite_whitelist_count;
//     //         indefinite_whitelist_count -= U256::from(1);
//     //         user_to_whitelist_status.insert(user, &whitelist_status);
//     //     }
//     //
//     //     let mut user_to_setter_indefinite_whitelist_status = self
//     //         .service_id_to_user_to_setter_to_indefinite_whitelist_status
//     //         .remove(service_id)
//     //         .expect("must have a service id");
//     //
//     //     let mut revoked = false;
//     //     if let Some(mut setter_indefinite_whitelist_status) =
//     //     user_to_setter_indefinite_whitelist_status.remove(user)
//     //     {
//     //         setter_indefinite_whitelist_status.insert(setter, &false);
//     //         user_to_setter_indefinite_whitelist_status
//     //             .insert(user, &setter_indefinite_whitelist_status);
//     //
//     //         revoked = true;
//     //     }
//     //
//     //     self.service_id_to_user_to_whitelist_status
//     //         .insert(service_id, &user_to_whitelist_status);
//     //
//     //     (revoked, indefinite_whitelist_count)
//     // }
// }