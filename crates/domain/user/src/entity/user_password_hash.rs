use crate::types::{PasswordHash, UserId};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(any(test, feature = "testing"), derive(::fake::Dummy))]
pub struct UserPasswordHash {
    pub user_id: UserId,
    pub hash: PasswordHash,
}
