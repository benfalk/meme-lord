use super::prelude::*;
use crate::entity::UserTagLink;
use crate::types::{MemeId, TagId};

#[derive(::derive_more::Debug, Clone, PartialEq, Eq)]
pub struct DeleteUserTagLink {
    pub meme_id: MemeId,
    pub tag_id: TagId,
}

impl Command for DeleteUserTagLink {
    type Value = ();
    type Error = DeleteUserTagLinkError;

    async fn exec(self, env: &impl EnvExt) -> Result<Self::Value, Self::Error> {
        let link = UserTagLink {
            meme_id: self.meme_id,
            tag_id: self.tag_id,
        };
        env.meme_repo().delete_user_tag_link(&link).await?;
        env.event_publisher().user_tag_link_deleted(&link).await?;
        Ok(())
    }
}

#[derive(Debug, ::thiserror::Error)]
pub enum DeleteUserTagLinkError {
    #[error("DeleteUserTagLink delete failed: {0}")]
    DeleteFailed(#[from] crate::port::meme_repo::DeleteUserTagLinkError),

    #[error(transparent)]
    Event(#[from] crate::port::event_publisher::PublishError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::support::env::*;

    #[tokio::test]
    #[rstest::rstest]
    async fn happy_path_works(mut mock_env: MockEnv) {
        let tag_id = TagId::generate();
        let meme_id = MemeId::generate();

        mock_env
            .meme_repo
            .expect_delete_user_tag_link()
            .withf(move |link| link.meme_id == meme_id && link.tag_id == tag_id)
            .return_once(|_| Box::pin(async { Ok(()) }));

        mock_env
            .event_publisher
            .expect_user_tag_link_deleted()
            .withf(move |link| link.meme_id == meme_id && link.tag_id == tag_id)
            .return_once(|_| Box::pin(async { Ok(()) }));

        let cmd = DeleteUserTagLink { meme_id, tag_id };
        cmd.exec(&mock_env).await.unwrap();
    }
}
