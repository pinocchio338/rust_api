use crate::{WrappedRoleDetail, WrappedUserWithRole};
use anchor_lang::accounts::account::Account;
use anchor_lang::prelude::{borsh::maybestd::collections::HashMap, msg};
use anchor_lang::solana_program::pubkey::Pubkey;
use api3_common::{ensure, AccessControlRegistry, Bytes32, Error};

const ROLE_SEED: &str = "role";
const MEMBER_SEED: &str = "member";

pub type UserWithRoleAccount<'info> = Account<'info, WrappedUserWithRole>;
pub type RoleDetailAccount<'info> = Account<'info, WrappedRoleDetail>;

/// Solana access control struct. This class mimics the implementation of
/// Openzeppelin's access control contract.
pub(crate) struct SolanaAccessControl<'account, 'info> {
    program_id: Pubkey,
    manager: Bytes32,
    admin_role_description: String,
    user_role_accounts: HashMap<Pubkey, &'account mut UserWithRoleAccount<'info>>,
    role_detail_accounts: HashMap<Pubkey, &'account mut RoleDetailAccount<'info>>,
}

impl SolanaAccessControl<'_, '_> {
    fn hash_role_with_who(
        &self,
        role: &Bytes32,
        who: &<SolanaAccessControl<'_, '_> as AccessControlRegistry>::Address,
    ) -> Pubkey {
        let (address, _) = Pubkey::find_program_address(
            &[ROLE_SEED.as_bytes(), role, MEMBER_SEED.as_bytes(), who],
            &self.program_id,
        );
        address
    }

    fn hash_role(&self, role: &Bytes32) -> Pubkey {
        let (address, _) =
            Pubkey::find_program_address(&[ROLE_SEED.as_bytes(), role], &self.program_id);
        address
    }
}

impl AccessControlRegistry for SolanaAccessControl<'_, '_> {
    type Address = Bytes32;

    fn manager(&self) -> &Self::Address {
        &self.manager
    }

    fn admin_role_description(&self) -> String {
        self.admin_role_description.clone()
    }

    fn has_role(&self, role: &Bytes32, who: &Self::Address) -> bool {
        let key = self.hash_role_with_who(role, who);
        self.user_role_accounts
            .get(&key)
            .map(|account| account.has_role)
            .unwrap_or(false)
    }

    fn grant_role(&mut self, role: &Bytes32, who: &Self::Address) {
        if !self.has_role(role, who) {
            let key = self.hash_role_with_who(role, who);
            match self.user_role_accounts.get_mut(&key) {
                None => {}
                Some(a) => {
                    a.has_role = true;
                }
            }
        }
    }

    fn get_role_admin(&self, role: &Bytes32) -> Option<Bytes32> {
        let key = self.hash_role(role);
        self.role_detail_accounts.get(&key).map(|a| a.admin_role)
    }

    fn set_role_admin(&mut self, role: &Bytes32, role_admin: Bytes32) {
        let previous_admin_role = self.get_role_admin(role);
        let key = self.hash_role(role);
        match self.role_detail_accounts.get_mut(&key) {
            None => {}
            Some(a) => {
                msg!(
                    "Role: {:?} admin changed from previous admin role: {:?} to admin role: {:?}",
                    role,
                    previous_admin_role,
                    role_admin
                );
                a.admin_role = role_admin;
            }
        }
    }

    fn renounce_role(
        &mut self,
        role: &Bytes32,
        account: &Self::Address,
        msg_sender: &Self::Address,
    ) -> Result<(), Error> {
        ensure!(account == msg_sender, Error::OnlyRenounceRolesForSelf)?;
        if self.has_role(role, account) {
            let key = self.hash_role_with_who(role, account);
            match self.user_role_accounts.get_mut(&key) {
                None => {}
                Some(a) => {
                    a.has_role = false;
                }
            }
        }
        Ok(())
    }
}

#[derive(Default)]
pub(crate) struct DummyAccessControl {
    manager: Bytes32,
}

impl AccessControlRegistry for DummyAccessControl {
    type Address = Bytes32;

    fn manager(&self) -> &Self::Address {
        &self.manager
    }

    fn admin_role_description(&self) -> String {
        String::from("hello")
    }

    fn has_role(&self, _role: &Bytes32, _who: &Self::Address) -> bool {
        true
    }

    fn grant_role(&mut self, _role: &Bytes32, _who: &Self::Address) {}

    fn get_role_admin(&self, _role: &Bytes32) -> Option<Bytes32> {
        Some(Bytes32::default())
    }

    fn set_role_admin(&mut self, _role: &Bytes32, _role_admin: Bytes32) {}

    fn renounce_role(
        &mut self,
        _role: &Bytes32,
        _account: &Self::Address,
        _msg_sender: &Self::Address,
    ) -> Result<(), Error> {
        Ok(())
    }
}
