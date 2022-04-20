use crate::{ensure, keccak_packed, Bytes32, Error, Token, Zero};

/// Roles that are known at dev time.
pub enum StaticRole {
    UnlimitedReaderRole,
    NameSetterRole,
}
/// The access control registry interface in the solidity contract
pub trait AccessControlRegistry {
    /// The address type for the chain
    type Address: AsRef<[u8]> + Zero;
    /// Default admin role, align with Openzepplin's definition
    const DEFAULT_ADMIN_ROLE: Bytes32 = [0; 32];
    const NAME_SETTER_ROLE_DESCRIPTION: &'static str = "Name setter";
    const UNLIMITED_READER_ROLE_DESCRIPTION: &'static str = "Unlimited reader";

    /// Get the manager of this registry
    fn manager(&self) -> &Self::Address;
    /// Admin role description
    fn admin_role_description(&self) -> String;
    /// Find the role by its name. Not in the original solidity contract
    /// Just for making it work in Rust
    fn find_static_role(&self, role: StaticRole) -> Bytes32 {
        match role {
            StaticRole::UnlimitedReaderRole => self.derive_role(
                self.derive_admin_role(self.manager()),
                Self::UNLIMITED_READER_ROLE_DESCRIPTION.parse().unwrap(),
            ),
            StaticRole::NameSetterRole => self.derive_role(
                self.derive_admin_role(self.manager()),
                Self::NAME_SETTER_ROLE_DESCRIPTION.parse().unwrap(),
            ),
        }
    }
    /// Checks if user has a particular role
    /// `role` The role to check
    /// `who` The address to check
    fn has_role(&self, role: &Bytes32, who: &Self::Address) -> bool;
    /// Grant role for the user
    /// `role` The role to grant
    /// `who` The address to grant role
    fn grant_role(&mut self, role: &Bytes32, who: &Self::Address);
    /// Get the admin role of role
    /// `role` The role to check
    fn get_role_admin(&self, role: &Bytes32) -> Option<Bytes32>;
    /// Set the role admin for a role
    /// `role` The role to grant
    /// `role_admin` The role admin
    fn set_role_admin(&mut self, role: &Bytes32, role_admin: Bytes32);
    /// Called by the account to renounce the role
    /// Override to disallow managers to renounce their root roles.
    /// `role` and `account` are not validated because
    /// `role` Role to be renounced
    /// `account` Account to renounce the role
    /// `msg_sender` The message sender
    fn renounce_role(
        &mut self,
        role: &Bytes32,
        account: &Self::Address,
        msg_sender: &Self::Address,
    ) -> Result<(), Error>;
    /// Initializes the manager by initializing its root role and
    /// granting it to them
    /// Anyone can initialize a manager. An uninitialized manager
    /// attempting to initialize a role will be initialized automatically.
    /// Once a manager is initialized, subsequent initializations have no
    /// effect.
    /// `manager` Manager address to be initialized
    fn initialize_manager(&mut self, manager: &Self::Address) -> Result<(), Error> {
        ensure!(!manager.is_zero(), Error::InvalidAddress)?;
        let root_role = RoleDeriver::derive_root_role(manager.as_ref());
        if !self.has_role(&root_role, manager) {
            self.grant_role(&root_role, manager);
        }
        Ok(())
    }
    /// Initializes a role by setting its admin role and grants it to
    /// the sender
    /// If the sender should not have the initialized role, they should
    /// explicitly renounce it after initializing it.
    /// Once a role is initialized, subsequent initializations have no effect
    /// other than granting the role to the sender.
    /// The sender must be a member of `admin_role`. `admin_role` value is not
    /// validated because the sender cannot have the `bytes32(0)` role.
    /// If the sender is an uninitialized manager that is initializing a role
    /// directly under their root role, manager initialization will happen
    /// automatically, which will grant the sender `admin_role` and allow them
    /// to initialize the role.
    /// `admin_role` Admin role to be assigned to the initialized role
    /// `description` Human-readable description of the initialized role
    /// `msg_sender` The message sender address
    fn initialize_role_and_grant_to_sender(
        &mut self,
        admin_role: Bytes32,
        description: String,
        msg_sender: &Self::Address,
    ) -> Result<Bytes32, Error> {
        ensure!(!description.is_empty(), Error::RoleDescriptionEmpty)?;
        let role = self.derive_role(admin_role, description);

        // AccessControl roles have `DEFAULT_ADMIN_ROLE` (i.e., `bytes32(0)`)
        // as their `admin_role` by default. No account in AccessControlRegistry
        // can possibly have that role, which means all initialized roles will
        // have non-default admin roles, and vice versa.
        if self.get_role_admin(&role) == Some(Self::DEFAULT_ADMIN_ROLE) {
            if admin_role == self.derive_root_role(msg_sender) {
                self.initialize_manager(msg_sender)?;
            }
            self.set_role_admin(&role, admin_role);
        }
        self.grant_role(&role, msg_sender);
        Ok(role)
    }
    /// Derives the admin role of the manager
    /// `manager` Manager address
    fn derive_admin_role(&self, manager: &Self::Address) -> Bytes32 {
        self.derive_role(
            self.derive_root_role(manager),
            self.admin_role_description(),
        )
    }
    /// Derives the root role of the manager
    /// `manager` Manager address
    fn derive_root_role(&self, manager: &Self::Address) -> Bytes32 {
        RoleDeriver::derive_root_role(manager.as_ref())
    }
    /// Derives the role using its admin role and description
    ///
    /// This implies that roles adminned by the same role cannot have the
    /// same description
    /// `admin_role` Admin role
    /// `description` Human-readable description of the role
    fn derive_role(&self, admin_role: Bytes32, description: String) -> Bytes32 {
        RoleDeriver::derive_role(admin_role, description)
    }
}

/// Contract that implements the AccessControlRegistry role derivation logic
///
/// If a contract interfaces with AccessControlRegistry and needs to
/// derive roles, it should inherit this contract instead of re-implementing
/// the logic
struct RoleDeriver {}

impl RoleDeriver {
    /// Derives the root role of the manager
    /// `manager` Manager address
    /// `rootRole` Root role
    pub fn derive_root_role(manager: &[u8]) -> Bytes32 {
        keccak_packed(&[Token::FixedBytes(manager.to_vec())])
    }

    /// Derives the role using its admin role and description
    ///
    /// This implies that roles adminned by the same role cannot have the
    /// same description
    /// `admin_role` Admin role
    /// `description` Human-readable description of the role
    pub fn derive_role(admin_role: Bytes32, description: String) -> Bytes32 {
        Self::derive_role_with_hash(admin_role, keccak_packed(&[Token::String(description)]))
    }

    /// Derives the role using its admin role and description hash
    ///
    /// This implies that roles adminned by the same role cannot have the
    /// same description
    /// `admin_role` Admin role
    /// `description` Hash of the human-readable description of the role
    pub fn derive_role_with_hash(admin_role: Bytes32, description_hash: Bytes32) -> Bytes32 {
        keccak_packed(&[
            Token::FixedBytes(admin_role.to_vec()),
            Token::FixedBytes(description_hash.to_vec()),
        ])
    }
}

#[cfg(test)]
mod tests {}
