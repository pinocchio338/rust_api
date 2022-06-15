mod types;
mod utils;
mod whitelist;

use crate::types::{Address, NearDataPoint};
use crate::utils::{
    msg_sender, Bytes32HashMap, DatapointHashMap, NearAccessControlRegistry, NearClock,
    SignatureVerify,
};
use crate::whitelist::{NearWhitelist, WhitelistStatus};
use api3_common::abi::{Token, Uint};
use api3_common::{
    keccak_packed, process_beacon_update, AccessControlRegistry, Bytes, Bytes32, Error,
    SignatureManger, StaticRole, WhitelistRolesWithManager, WhitelistWithManager,
};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{collections::LookupMap, near_bindgen};
use std::fmt::Debug;

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct DapiServer {
    initialized: bool,
    /// Data point related storage
    data_points: LookupMap<Bytes32, NearDataPoint>,
    name_hash_to_data_point_id: LookupMap<Bytes32, Bytes32>,

    /// Access control related storage
    manager: Address,
    admin_role_description: String,
    role_membership: LookupMap<Bytes32, bool>,
    role_admin: LookupMap<Bytes32, Address>,

    service_id_to_user_to_whitelist_status: LookupMap<Bytes32, WhitelistStatus>,
    service_id_to_user_to_setter_to_indefinite_whitelist_status: LookupMap<Bytes32, bool>,
}

impl Default for DapiServer {
    fn default() -> Self {
        let data_points = LookupMap::new(b'd');
        let name_hash_to_data_point_id = LookupMap::new(b'n');

        let role_membership = LookupMap::new(b'm');
        let role_admin = LookupMap::new(b'a');

        let service_id_to_user_to_whitelist_status = LookupMap::new(b's');
        let service_id_to_user_to_setter_to_indefinite_whitelist_status = LookupMap::new(b'b');
        Self {
            initialized: false,
            data_points,
            name_hash_to_data_point_id,
            manager: Address(Bytes32::default()),
            admin_role_description: String::from("admin role"),
            role_membership,
            role_admin,
            service_id_to_user_to_whitelist_status,
            service_id_to_user_to_setter_to_indefinite_whitelist_status,
        }
    }
}

#[near_bindgen]
impl DapiServer {
    /// The initializer of the contract
    pub fn initialize(&mut self) {
        ensure!(!self.initialized, Error::AlreadyInitialized);

        let manager = msg_sender();
        let mut access = NearAccessControlRegistry::requires_write(
            manager.clone(),
            self.admin_role_description.clone(),
            &mut self.role_membership,
            &mut self.role_admin,
        );
        access
            .grant_role(
                &NearAccessControlRegistry::DEFAULT_ADMIN_ROLE,
                &msg_sender(),
            )
            .expect("initialization failed");

        self.manager = manager;
        self.initialized = true;
    }

    // ================== Access Control ====================
    pub fn roles(&self) -> (Bytes32, Bytes32) {
        let access = NearAccessControlRegistry::read_only(
            self.manager.clone(),
            self.admin_role_description.clone(),
            &self.role_membership,
            &self.role_admin,
        );
        (
            access.find_static_role(StaticRole::UnlimitedReaderRole),
            access.find_static_role(StaticRole::NameSetterRole),
        )
    }

    /// Renounce `role` to `who`
    pub fn renounce_role(&mut self, role: Bytes32, who: Bytes32) {
        let mut access = NearAccessControlRegistry::requires_write(
            self.manager.clone(),
            self.admin_role_description.clone(),
            &mut self.role_membership,
            &mut self.role_admin,
        );
        let r = access.renounce_role(&role, &Address(who));
        near_check_result(r)
    }

    /// Revoke `role` to `who`
    pub fn revoke_role(&mut self, role: Bytes32, who: Bytes32) {
        let mut access = NearAccessControlRegistry::requires_write(
            self.manager.clone(),
            self.admin_role_description.clone(),
            &mut self.role_membership,
            &mut self.role_admin,
        );

        let role_admin = access
            .get_role_admin(&role)
            .unwrap_or(NearAccessControlRegistry::DEFAULT_ADMIN_ROLE);

        ensure!(
            access.only_role(&role_admin, &msg_sender()).is_ok(),
            Error::NotAuthorized
        );

        let r = access.revoke_role(&role, &Address(who));
        near_check_result(r)
    }

    /// Grants `role` to `who`
    pub fn grant_role(&mut self, role: Bytes32, who: Bytes32) {
        let mut access = NearAccessControlRegistry::requires_write(
            self.manager.clone(),
            self.admin_role_description.clone(),
            &mut self.role_membership,
            &mut self.role_admin,
        );

        ensure!(
            access
                .only_role(
                    &NearAccessControlRegistry::DEFAULT_ADMIN_ROLE,
                    &msg_sender()
                )
                .is_ok(),
            Error::NotAuthorized
        );

        let r = access.grant_role(&role, &Address(who));
        near_check_result(r)
    }

    /// Checks if `who` has `role`
    pub fn has_role(&self, role: Bytes32, who: Bytes32) -> bool {
        let access = NearAccessControlRegistry::read_only(
            self.manager.clone(),
            self.admin_role_description.clone(),
            &self.role_membership,
            &self.role_admin,
        );
        access.has_role(&role, &Address(who))
    }

    // ================== Datapoint ====================
    /// Updates a Beacon using data signed by the respective Airnode,
    /// without requiring a request or subscription
    ///
    /// # Arguments
    ///
    /// * `airnode` Airnode address
    /// * `template_id` Template ID
    /// * `timestamp` Timestamp used in the signature
    /// * `data` Response data (an `int256` encoded in contract ABI)
    /// * `signature` Template ID, a timestamp and the response data signed by the Airnode address
    pub fn update_beacon_with_signed_data(
        &mut self,
        airnode: Bytes,
        template_id: Bytes32,
        timestamp: Bytes32,
        data: Vec<u8>,
        signature: Vec<u8>,
    ) {
        // create the utility structs
        let mut storage = DatapointHashMap::requires_write(&mut self.data_points);

        // perform signature verification
        let message = keccak_packed(&[
            Token::FixedBytes(template_id.to_vec()),
            Token::Uint(Uint::from_big_endian(&timestamp)),
            Token::Bytes(data.clone()),
        ]);

        if !SignatureVerify::verify(&airnode, &message, &signature) {
            near_sdk::env::panic("Signature verification wrong".as_ref());
        }

        let beacon_id = api3_common::derive_beacon_id(airnode.to_vec(), template_id);
        let r = process_beacon_update(
            &mut storage,
            beacon_id,
            Uint::from_big_endian(&timestamp),
            data,
        );
        near_check_result(r)
    }

    /// Updates the dAPI that is specified by the beacon IDs
    ///
    /// # Arguments
    ///
    /// * `beacon_ids` Beacon IDs
    pub fn update_dapi_with_beacons(&mut self, beacon_ids: Vec<Bytes32>) -> Bytes32 {
        let mut storage = DatapointHashMap::requires_write(&mut self.data_points);
        let r = api3_common::update_dapi_with_beacons(&mut storage, &beacon_ids);
        near_check_result(r)
    }

    /// Updates a dAPI using data signed by the respective Airnodes
    /// without requiring a request or subscription. The beacons for which the
    /// signature is omitted will be read from the storage.
    ///
    /// # Arguments
    ///
    /// * `airnodes` Airnode addresses
    /// * `template_ids` Template IDs
    /// * `timestamps` Timestamps used in the signatures
    /// * `data` Response data (an `int256` encoded in contract ABI per Beacon)
    /// * `signatures` Template ID, a timestamp and the response data signed by the respective Airnode address per Beacon
    pub fn update_dapi_with_signed_data(
        &mut self,
        airnodes: Vec<Bytes>,
        template_ids: Vec<Bytes32>,
        timestamps: Vec<Bytes32>,
        data: Vec<Bytes>,
        signatures: Vec<Bytes>,
    ) -> Bytes32 {
        let mut storage = DatapointHashMap::requires_write(&mut self.data_points);
        let clock = NearClock::new(nanoseconds_to_seconds(near_sdk::env::block_timestamp()));

        let r = api3_common::update_dapi_with_signed_data::<_, SignatureVerify, _>(
            &mut storage,
            &clock,
            airnodes,
            template_ids,
            timestamps,
            data,
            signatures,
        );
        near_check_result(r)
    }

    /// Sets the data point ID the name points to.
    /// While a data point ID refers to a specific Beacon or dAPI, names
    /// provide a more abstract interface for convenience. This means a name
    /// that was pointing at a Beacon can be pointed to a dAPI, then another
    /// dAPI, etc.
    ///
    /// # Arguments
    ///
    /// * `name` Human-readable name
    /// * `datapoint_id` Data point ID the name will point to
    pub fn set_name(&mut self, name: Bytes32, datapoint_id: Bytes32) {
        let mut storage = Bytes32HashMap::requires_write(&mut self.name_hash_to_data_point_id);
        let access = NearAccessControlRegistry::read_only(
            self.manager.clone(),
            self.admin_role_description.clone(),
            &self.role_membership,
            &self.role_admin,
        );
        let r = api3_common::set_name(name, datapoint_id, &msg_sender(), &access, &mut storage);
        near_check_result(r)
    }

    /// Returns the data point ID the name is set to
    /// `name` Name
    pub fn name_to_data_point_id(&self, name: Bytes32) -> Option<Bytes32> {
        self.name_hash_to_data_point_id
            .get(&keccak_packed(&[Token::FixedBytes(name.to_vec())]))
    }

    /// Derives the beacon set ID from the beacon IDs
    /// Notice that `encode()` is used over `encode_packed()`
    /// Returns the derived dapi id
    ///
    /// # Arguments
    ///
    /// * `beacon_ids` Beacon IDs
    pub fn derive_beacon_set_id(&self, beacon_ids: Vec<Bytes32>) -> Bytes32 {
        api3_common::derive_dapi_id(&beacon_ids)
    }

    /// Derives the beacon id based on the `airnode` and `templated_id`
    /// Returns the beacon id
    ///
    /// # Arguments
    ///
    /// * `airnode` Airnode address
    /// * `template_id` Template ID
    pub fn derive_beacon_id(&self, airnode: Bytes, template_id: Bytes32) -> Bytes32 {
        api3_common::derive_beacon_id(airnode, template_id)
    }

    /// Reads the data point with ID
    ///
    /// # Arguments
    ///
    /// * `data_point_id` Data point ID
    pub fn read_with_data_point_id(&self, data_point_id: Bytes32) -> (Bytes32, u32) {
        let storage = DatapointHashMap::read_only(&self.data_points);
        let access = NearAccessControlRegistry::read_only(
            self.manager.clone(),
            self.admin_role_description.clone(),
            &self.role_membership,
            &self.role_admin,
        );
        let whitelist = NearWhitelist::read_only(
            &access,
            &self.service_id_to_user_to_whitelist_status,
            &self.service_id_to_user_to_setter_to_indefinite_whitelist_status,
        );

        let r = api3_common::read_with_data_point_id(
            &data_point_id,
            &msg_sender(),
            &storage,
            &access,
            &whitelist,
        )
        .map(|(a, n)| {
            let mut v = [0u8; 32];
            a.to_big_endian(&mut v);
            (v, n)
        });
        near_check_result(r)
    }

    /// Reads the data point with name
    /// The read data point may belong to a Beacon or dAPI. The reader
    /// must be whitelisted for the hash of the data point name.
    ///
    /// # Arguments
    ///
    /// * `name` Data point name
    pub fn read_with_name(&self, name: Bytes32) -> (Bytes32, u32) {
        let dp_s = DatapointHashMap::read_only(&self.data_points);
        let nh_s = Bytes32HashMap::read_only(&self.name_hash_to_data_point_id);
        let access = NearAccessControlRegistry::read_only(
            self.manager.clone(),
            self.admin_role_description.clone(),
            &self.role_membership,
            &self.role_admin,
        );
        let whitelist = NearWhitelist::read_only(
            &access,
            &self.service_id_to_user_to_whitelist_status,
            &self.service_id_to_user_to_setter_to_indefinite_whitelist_status,
        );
        let r = api3_common::read_with_name(name, &msg_sender(), &dp_s, &nh_s, &access, &whitelist)
            .map(|(a, n)| {
                let mut v = [0u8; 32];
                a.to_big_endian(&mut v);
                (v, n)
            });
        near_check_result(r)
    }

    /// Returns if a reader can read the data point
    ///
    /// # Arguments
    ///
    /// * `data_point_id` Data point ID (or data point name hash)
    /// * `reader` Reader address as raw bytes
    pub fn reader_can_read_data_point(&self, data_point_id: Bytes32, reader: Bytes32) -> bool {
        let access = NearAccessControlRegistry::read_only(
            self.manager.clone(),
            self.admin_role_description.clone(),
            &self.role_membership,
            &self.role_admin,
        );
        let whitelist = NearWhitelist::read_only(
            &access,
            &self.service_id_to_user_to_whitelist_status,
            &self.service_id_to_user_to_setter_to_indefinite_whitelist_status,
        );
        let reader = Address(reader);
        api3_common::reader_can_read_data_point(&data_point_id, &reader, &access, &whitelist)
    }

    pub fn data_feed_id_to_reader_to_setter_to_indefinite_whitelist_status(
        &self,
        data_feed_id: Bytes32,
        reader: Bytes32,
        setter: Bytes32,
    ) -> bool {
        let access = NearAccessControlRegistry::read_only(
            self.manager.clone(),
            self.admin_role_description.clone(),
            &self.role_membership,
            &self.role_admin,
        );
        let whitelist = NearWhitelist::read_only(
            &access,
            &self.service_id_to_user_to_whitelist_status,
            &self.service_id_to_user_to_setter_to_indefinite_whitelist_status,
        );
        whitelist
            .data_feed_id_to_reader_to_setter_to_indefinite_whitelist_status(
                &data_feed_id,
                &reader,
                &setter,
            )
            .unwrap_or(false)
    }

    /// Returns the detailed whitelist status of the reader for the data feed
    ///
    /// # Arguments
    ///
    /// * `data_feed_id` The data feed id
    /// * `reader` Reader address
    pub fn data_feed_id_to_whitelist_status(
        &self,
        data_feed_id: Bytes32,
        reader: Bytes32,
    ) -> Option<(u64, Bytes32)> {
        let access = NearAccessControlRegistry::read_only(
            self.manager.clone(),
            self.admin_role_description.clone(),
            &self.role_membership,
            &self.role_admin,
        );
        let whitelist = NearWhitelist::read_only(
            &access,
            &self.service_id_to_user_to_whitelist_status,
            &self.service_id_to_user_to_setter_to_indefinite_whitelist_status,
        );
        whitelist.data_feed_id_to_whitelist_status(&data_feed_id, &reader)
    }

    /// Extends the expiration of the temporary whitelist of `user` to
    /// be able to use the service with `service_id` if the sender has the
    /// whitelist expiration extender role
    ///
    /// # Arguments
    ///
    /// * `service_id` Service ID
    /// * `user` User address
    /// * `expiration_timestamp` Timestamp at which the temporary whitelist will expire
    pub fn extend_whitelist_expiration(
        &mut self,
        service_id: Bytes32,
        user: Bytes32,
        expiration_timestamp: u64,
    ) {
        let access = NearAccessControlRegistry::read_only(
            self.manager.clone(),
            self.admin_role_description.clone(),
            &self.role_membership,
            &self.role_admin,
        );
        let mut whitelist = NearWhitelist::requires_write(
            &access,
            &mut self.service_id_to_user_to_whitelist_status,
            &mut self.service_id_to_user_to_setter_to_indefinite_whitelist_status,
        );
        whitelist.extend_whitelist_expiration(&service_id, &Address(user), expiration_timestamp)
    }

    /// Sets the expiration of the temporary whitelist of `user` to be
    /// able to use the service with `service_id` if the sender has the
    /// whitelist expiration setter role
    ///
    /// # Arguments
    ///
    /// * `service_id` Service ID
    /// * `user` User address
    /// * `expiration_timestamp` Timestamp at which the temporary whitelist will expire
    pub fn set_whitelist_expiration(
        &mut self,
        service_id: Bytes32,
        user: Bytes32,
        expiration_timestamp: u64,
    ) {
        let access = NearAccessControlRegistry::read_only(
            self.manager.clone(),
            self.admin_role_description.clone(),
            &self.role_membership,
            &self.role_admin,
        );
        let mut whitelist = NearWhitelist::requires_write(
            &access,
            &mut self.service_id_to_user_to_whitelist_status,
            &mut self.service_id_to_user_to_setter_to_indefinite_whitelist_status,
        );
        whitelist.set_whitelist_expiration(&service_id, &Address(user), expiration_timestamp)
    }

    /// Sets the indefinite whitelist status of `user` to be able to
    /// use the service with `service_id` if the sender has the indefinite whitelister role
    ///
    /// # Arguments
    ///
    /// * `service_id` Service ID
    /// * `user` User address
    /// * `status` Indefinite whitelist status
    pub fn set_indefinite_whitelist_status(
        &mut self,
        service_id: Bytes32,
        user: Bytes32,
        status: bool,
    ) -> Bytes32 {
        let access = NearAccessControlRegistry::read_only(
            self.manager.clone(),
            self.admin_role_description.clone(),
            &self.role_membership,
            &self.role_admin,
        );
        let mut whitelist = NearWhitelist::requires_write(
            &access,
            &mut self.service_id_to_user_to_whitelist_status,
            &mut self.service_id_to_user_to_setter_to_indefinite_whitelist_status,
        );
        let r = whitelist.set_indefinite_whitelist_status(&service_id, &Address(user), status);
        Bytes32::from(r)
    }

    /// Revokes the indefinite whitelist status granted to the user for
    /// the service by a specific account
    ///
    /// # Arguments
    ///
    /// * `service_id` Service ID
    /// * `user` User address
    /// * `setter` Setter address
    pub fn revoke_indefinite_whitelist_status(
        &mut self,
        service_id: Bytes32,
        user: Bytes32,
        setter: Bytes32,
    ) -> (bool, Bytes32) {
        let access = NearAccessControlRegistry::read_only(
            self.manager.clone(),
            self.admin_role_description.clone(),
            &self.role_membership,
            &self.role_admin,
        );
        let mut whitelist = NearWhitelist::requires_write(
            &access,
            &mut self.service_id_to_user_to_whitelist_status,
            &mut self.service_id_to_user_to_setter_to_indefinite_whitelist_status,
        );
        let (revoked, r) = whitelist.revoke_indefinite_whitelist_status(
            &service_id,
            &Address(user),
            &Address(setter),
        );
        (revoked, Bytes32::from(r))
    }

    pub fn whitelist_expiration_extender_role(&self) -> Bytes32 {
        let access = NearAccessControlRegistry::read_only(
            self.manager.clone(),
            self.admin_role_description.clone(),
            &self.role_membership,
            &self.role_admin,
        );
        let whitelist = NearWhitelist::read_only(
            &access,
            &self.service_id_to_user_to_whitelist_status,
            &self.service_id_to_user_to_setter_to_indefinite_whitelist_status,
        );
        whitelist.whitelist_expiration_extender_role()
    }

    pub fn whitelist_expiration_setter_role(&self) -> Bytes32 {
        let access = NearAccessControlRegistry::read_only(
            self.manager.clone(),
            self.admin_role_description.clone(),
            &self.role_membership,
            &self.role_admin,
        );
        let whitelist = NearWhitelist::read_only(
            &access,
            &self.service_id_to_user_to_whitelist_status,
            &self.service_id_to_user_to_setter_to_indefinite_whitelist_status,
        );
        whitelist.whitelist_expiration_setter_role()
    }

    pub fn indefinite_whitelister_role(&self) -> Bytes32 {
        let access = NearAccessControlRegistry::read_only(
            self.manager.clone(),
            self.admin_role_description.clone(),
            &self.role_membership,
            &self.role_admin,
        );
        let whitelist = NearWhitelist::read_only(
            &access,
            &self.service_id_to_user_to_whitelist_status,
            &self.service_id_to_user_to_setter_to_indefinite_whitelist_status,
        );
        whitelist.indefinite_whitelister_role()
    }
}

fn nanoseconds_to_seconds(nano: u64) -> u32 {
    (nano / (1e9 as u64)) as u32
}

fn near_check_result<T: Debug>(r: Result<T, Error>) -> T {
    if r.is_ok() {
        r.unwrap()
    } else {
        near_sdk::env::panic((&format!("Invalid request: {:?}", r.unwrap_err())).as_ref())
    }
}
