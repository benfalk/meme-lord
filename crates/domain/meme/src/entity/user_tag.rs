use crate::types::{TagId, TagName};
use ::identity::UserId;

/// Name unique to the owner which can be used to identify a tag id.
/// This tag is then linked to other memes by their ids, allowing users
/// to group memes together by tags. A tag can be used to find all memes
/// tagged with it.
///
/// ---
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(any(test, feature = "testing"), derive(::fake::Dummy))]
pub struct UserTag {
    pub id: TagId,
    pub owner_id: UserId,
    pub name: TagName,
}
