use super::prelude::*;
use crate::types::TagId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteUserTag {
    pub tag_id: TagId,
}

impl Command for DeleteUserTag {
    type Value = ();
    type Error = DeleteUserTagError;

    async fn exec(self, env: &impl EnvExt) -> Result<Self::Value, Self::Error> {
        env.meme_repo().delete_user_tag_by_id(&self.tag_id).await?;
        env.event_publisher().user_tag_deleted(self.tag_id).await?;
        Ok(())
    }
}

#[derive(Debug, ::thiserror::Error)]
pub enum DeleteUserTagError {
    #[error("DeleteUserTag delete failed: {0}")]
    DeleteFailed(#[from] crate::port::meme_repo::DeleteUserTagByIdError),

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

        mock_env
            .meme_repo
            .expect_delete_user_tag_by_id()
            .withf(move |id| *id == tag_id)
            .return_once(|_| Box::pin(async { Ok(()) }));

        mock_env
            .event_publisher
            .expect_user_tag_deleted()
            .withf(move |id| *id == tag_id)
            .return_once(|_| Box::pin(async { Ok(()) }));

        let cmd = DeleteUserTag { tag_id };
        cmd.exec(&mock_env).await.unwrap();
    }
}
