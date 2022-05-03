use api3_common::{Bytes32, Whitelist, U256};

#[derive(Default)]
pub(crate) struct DummyWhitelist;

impl Whitelist for DummyWhitelist {
    type Address = Bytes32;
    type U256 = U256;

    fn user_is_whitelisted(&self, _service_id: &Bytes32, _user: &Self::Address) -> bool {
        true
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
    ) -> Self::U256 {
        Self::U256::default()
    }

    fn revoke_indefinite_whitelist_status(
        &mut self,
        _service_id: &Bytes32,
        _user: &Self::Address,
        _setter: &Self::Address,
    ) -> (bool, Self::U256) {
        (true, Self::U256::default())
    }
}
