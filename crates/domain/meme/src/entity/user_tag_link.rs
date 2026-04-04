use crate::types::{MemeId, TagId};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(any(test, feature = "testing"), derive(::fake::Dummy))]
pub struct UserTagLink {
    pub tag_id: TagId,
    pub meme_id: MemeId,
}
