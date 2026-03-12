use crate::types::{UserId, Username};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(any(test, feature = "testing"), derive(::fake::Dummy))]
pub struct User {
    pub id: UserId,
    pub username: Username,
}
