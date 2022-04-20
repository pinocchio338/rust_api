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
    /// `service_id` Service ID
    /// `user` User address
    fn user_is_whitelisted(&self, service_id: &Bytes32, user: &Self::Address) -> bool;
}
