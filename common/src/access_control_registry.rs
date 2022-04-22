use crate::Bytes32;
use crate::Zero;

pub trait AccessControl {}

pub trait RoleDriver {}

/// @title Contract that allows users to manage independent, tree-shaped access
/// control tables
/// @notice Multiple contracts can refer to this contract to check if their
/// users have granted accounts specific roles. Therefore, it aims to keep all
/// access control roles of its users in this single contract.
/// @dev Each user is called a "manager", and is the only member of their root
/// role. Starting from this root role, they can create an arbitrary tree of
/// roles and grant these to accounts. Each role has a description, and roles
/// adminned by the same role cannot have the same description.
pub trait AccessControlRegistry: RoleDriver + AccessControl {
    type Address: AsRef<[u8]> + Zero;

    /// @notice Initializes the manager by initializing its root role and
    /// granting it to them
    /// @dev Anyone can initialize a manager. An uninitialized manager
    /// attempting to initialize a role will be initialized automatically.
    /// Once a manager is initialized, subsequent initializations have no
    /// effect.
    /// @param manager Manager address to be initialized
    fn initialize_manager(&mut self, manager: &Self::Address);

    /// @notice Called by the account to renounce the role
    /// @dev Overriden to disallow managers to renounce their root roles.
    /// `role` and `account` are not validated because
    /// `AccessControl.renounceRole` will revert if either of them is zero.
    /// @param role Role to be renounced
    /// @param account Account to renounce the role
    fn renounce_role(&mut self, role: &Bytes32, account: &Self::Address);
}
