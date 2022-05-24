use crate::types::{Address, NearDataPoint};
use api3_common::abi::Token;
use api3_common::{
    keccak_packed, AccessControlRegistry, AccessControlRegistryAdminnedWithManager, Bytes32,
    DataPoint, Error, RoleDeriver, SignatureManger, Storage, TimestampChecker,
};
use ed25519_dalek::Verifier;
use near_sdk::collections::LookupMap;

/// Read write privilege
pub(crate) enum ReadWrite<'a, T> {
    ReadOnly(&'a T),
    Write(&'a mut T),
}

/// The utility struct for handling Near storage so that
/// we can use the code in `api3_common` for all the processing
pub(crate) struct DatapointHashMap<'account> {
    map: ReadWrite<'account, LookupMap<Bytes32, NearDataPoint>>,
}

impl<'account> DatapointHashMap<'account> {
    pub fn requires_write(map: &'account mut LookupMap<Bytes32, NearDataPoint>) -> Self {
        Self {
            map: ReadWrite::Write(map),
        }
    }

    pub fn read_only(map: &'account LookupMap<Bytes32, NearDataPoint>) -> Self {
        Self {
            map: ReadWrite::ReadOnly(map),
        }
    }
}

impl<'account> Storage<DataPoint> for DatapointHashMap<'account> {
    fn get(&self, k: &Bytes32) -> Option<DataPoint> {
        match &self.map {
            ReadWrite::ReadOnly(a) => match (*a).get(k) {
                Some(d) => Some(d.into()),
                None => Some(DataPoint::default()),
            },
            ReadWrite::Write(a) => match (*a).get(k) {
                Some(d) => Some(d.into()),
                None => Some(DataPoint::default()),
            },
        }
    }

    fn store(&mut self, k: Bytes32, datapoint: DataPoint) {
        let m = match &mut self.map {
            ReadWrite::ReadOnly(_) => panic!("wrong privilege"),
            ReadWrite::Write(m) => m,
        };
        if (*m).contains_key(&k) {
            (*m).remove(&k);
        }
        (*m).insert(&k, &NearDataPoint::from(datapoint));
    }
}

/// The utility struct for handling Near storage so that
/// we can use the code in `api3_common` for all the processing
pub(crate) struct Bytes32HashMap<'account> {
    map: ReadWrite<'account, LookupMap<Bytes32, Bytes32>>,
}

impl<'account> Bytes32HashMap<'account> {
    pub fn requires_write(map: &'account mut LookupMap<Bytes32, Bytes32>) -> Self {
        Self {
            map: ReadWrite::Write(map),
        }
    }

    pub fn read_only(map: &'account LookupMap<Bytes32, Bytes32>) -> Self {
        Self {
            map: ReadWrite::ReadOnly(map),
        }
    }
}

impl<'account> Storage<Bytes32> for Bytes32HashMap<'account> {
    fn get(&self, k: &Bytes32) -> Option<Bytes32> {
        match &self.map {
            ReadWrite::ReadOnly(a) => (*a).get(k),
            ReadWrite::Write(a) => (*a).get(k),
        }
    }

    fn store(&mut self, k: Bytes32, data: Bytes32) {
        let m = match &mut self.map {
            ReadWrite::ReadOnly(_) => panic!("wrong privilege"),
            ReadWrite::Write(m) => m,
        };
        if (*m).contains_key(&k) {
            (*m).remove(&k);
        }
        (*m).insert(&k, &data);
    }
}

/// Utility function for signature verification for Near so that we can use
/// `api3_common` package for the functions
pub(crate) struct SignatureVerify;

impl SignatureManger for SignatureVerify {
    fn verify(key: &[u8], message: &[u8], signature: &[u8]) -> bool {
        let signature = ed25519_dalek::Signature::try_from(signature)
            .expect("Signature should be a valid array of 64 bytes");

        let public_key =
            ed25519_dalek::PublicKey::from_bytes(key).expect("Invalid public key passed");

        public_key.verify(message, &signature).is_ok()
    }
}

pub(crate) struct NearClock {
    current_timestamp: u32,
}

impl NearClock {
    pub fn new(current_timestamp: u32) -> Self {
        Self { current_timestamp }
    }
}

impl TimestampChecker for NearClock {
    fn current_timestamp(&self) -> u32 {
        self.current_timestamp
    }
}

pub(crate) fn msg_sender() -> Address {
    let sender = near_sdk::env::predecessor_account_id();
    let sender_bytes = sender.as_bytes();
    let mut v = Bytes32::default();
    v[0..sender_bytes.len()].copy_from_slice(sender_bytes);
    Address(v)
}

pub(crate) struct NearAccessControlRegistry<'a> {
    manager: Address,
    admin_role_description: String,
    role_membership: ReadWrite<'a, LookupMap<Bytes32, bool>>,
    role_admin: ReadWrite<'a, LookupMap<Bytes32, Address>>,
}

impl<'a> NearAccessControlRegistry<'a> {
    pub fn requires_write(
        manager: Address,
        admin_role_description: String,
        role_membership: &'a mut LookupMap<Bytes32, bool>,
        role_admin: &'a mut LookupMap<Bytes32, Address>,
    ) -> Self {
        Self {
            manager,
            admin_role_description,
            role_membership: ReadWrite::Write(role_membership),
            role_admin: ReadWrite::Write(role_admin),
        }
    }

    pub fn read_only(
        manager: Address,
        admin_role_description: String,
        role_membership: &'a LookupMap<Bytes32, bool>,
        role_admin: &'a LookupMap<Bytes32, Address>,
    ) -> Self {
        Self {
            manager,
            admin_role_description,
            role_membership: ReadWrite::ReadOnly(role_membership),
            role_admin: ReadWrite::ReadOnly(role_admin),
        }
    }

    fn hash_membership(role: &Bytes32, who: &Address) -> Bytes32 {
        keccak_packed(&[
            Token::FixedBytes(role.to_vec()),
            Token::FixedBytes(who.as_ref().to_vec()),
        ])
    }
}

impl<'a> AccessControlRegistryAdminnedWithManager for NearAccessControlRegistry<'a> {
    type Address = Address;

    fn manager(&self) -> &Self::Address {
        &self.manager
    }

    fn admin_role_description(&self) -> String {
        self.admin_role_description.clone()
    }

    fn admin_role_description_hash(&self) -> Bytes32 {
        keccak_packed(&[Token::String(self.admin_role_description())])
    }

    fn admin_role(&self) -> Bytes32 {
        RoleDeriver::derive_role(
            RoleDeriver::derive_root_role(&self.manager.0),
            self.admin_role_description(),
        )
    }
}

impl<'a> AccessControlRegistry for NearAccessControlRegistry<'a> {
    fn has_role(&self, role: &Bytes32, who: &Self::Address) -> bool {
        let hash = Self::hash_membership(role, who);
        match &self.role_membership {
            ReadWrite::ReadOnly(m) => m.contains_key(&hash),
            ReadWrite::Write(m) => m.contains_key(&hash),
        }
    }

    fn grant_role(&mut self, role: &Bytes32, who: &Self::Address) -> Result<(), Error> {
        let hash = Self::hash_membership(role, who);
        match &mut self.role_membership {
            ReadWrite::ReadOnly(_) => panic!("wrong privilege"),
            ReadWrite::Write(m) => {
                (*m).remove(&hash);
                (*m).insert(&hash, &true);
            }
        };
        Ok(())
    }

    fn get_role_admin(&self, role: &Bytes32) -> Option<Bytes32> {
        if *role == Self::DEFAULT_ADMIN_ROLE {
            return Some(Self::DEFAULT_ADMIN_ROLE);
        }
        match &self.role_admin {
            ReadWrite::ReadOnly(a) => (*a).get(role).map(Bytes32::from),
            ReadWrite::Write(a) => (*a).get(role).map(Bytes32::from),
        }
    }

    fn set_role_admin(&mut self, role: &Bytes32, role_admin: Bytes32) -> Result<(), Error> {
        let a = match &mut self.role_admin {
            ReadWrite::ReadOnly(_) => panic!("wrong privilege"),
            ReadWrite::Write(a) => a,
        };
        (*a).remove(role);
        (*a).insert(role, &Address(role_admin));
        Ok(())
    }

    fn renounce_role(&mut self, role: &Bytes32, account: &Self::Address) -> Result<(), Error> {
        // let sender = msg_sender();
        // api3_common::ensure!(*account == sender, Error::NotAuthorized)?;
        let hash = Self::hash_membership(role, account);

        let m = match &mut self.role_membership {
            ReadWrite::ReadOnly(_) => panic!("wrong privilege"),
            ReadWrite::Write(m) => m,
        };
        (*m).remove(&hash);
        Ok(())
    }
}

/// NEAR contract calls on the panic interface for errors
#[macro_export]
macro_rules! ensure {
    ( $x:expr, $y:expr ) => {{
        if !$x {
            near_sdk::env::panic(format!("{:?}", $y).as_bytes())
        }
    }};
}

/// a convenient way to call to the NEAR's blockchain panic
#[macro_export]
macro_rules! error_panic {
    ( $y:expr ) => {{
        near_sdk::env::panic(format!("{}", $y).as_bytes())
    }};
}
