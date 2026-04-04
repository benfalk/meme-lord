use super::prelude::*;
use crate::entity::UserTag;
use crate::types::TagName;
use ::identity::UserId;

#[derive(::derive_more::Debug, Clone, PartialEq, Eq)]
pub struct CreateUserTag {
    pub owner_id: UserId,
    pub name: TagName,
}

impl Command for CreateUserTag {
    type Value = UserTag;
    type Error = CreateUserTagError;

    async fn exec(self, env: &impl EnvExt) -> Result<Self::Value, Self::Error> {
        let tag = UserTag {
            id: env.id_generator().generate_tag_id().await?,
            owner_id: self.owner_id,
            name: self.name,
        };
        env.meme_repo().insert_user_tag(&tag).await?;
        env.event_publisher().user_tag_created(&tag).await?;
        Ok(tag)
    }
}

#[derive(Debug, ::thiserror::Error)]
pub enum CreateUserTagError {
    #[error(transparent)]
    IdGenerattion(#[from] crate::port::id_generator::GenerateTagIdError),

    #[error("Tag name already exists: {name}")]
    NameAlreadyExists { name: TagName },

    #[error("CreateUserTag insert failed: {0}")]
    InsertFailed(crate::port::meme_repo::InsertUserTagError),

    #[error(transparent)]
    Event(#[from] crate::port::event_publisher::PublishError),
}

mod impls {
    use super::*;
    use crate::port::meme_repo::InsertUserTagError;

    impl From<InsertUserTagError> for CreateUserTagError {
        fn from(value: InsertUserTagError) -> Self {
            match value {
                InsertUserTagError::NameTaken { name } => {
                    CreateUserTagError::NameAlreadyExists { name }
                }
                any => CreateUserTagError::InsertFailed(any),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::port::meme_repo::InsertUserTagError;
    use crate::support::env::*;
    use crate::types::TagId;

    #[tokio::test]
    #[rstest::rstest]
    async fn happy_path_works(mut mock_env: MockEnv) {
        let tag_id = TagId::generate();
        let owner_id = UserId::generate();
        let name = TagName::from("test");

        mock_env
            .id_generator
            .expect_generate_tag_id()
            .return_once(move || {
                let tag_id = tag_id;
                Box::pin(async move { Ok(tag_id) })
            });

        mock_env
            .meme_repo
            .expect_insert_user_tag()
            .withf(move |tag| {
                assert_eq!(tag.id, tag_id);
                assert_eq!(tag.owner_id, owner_id);
                assert_eq!(tag.name, "test");
                true
            })
            .return_once(|_| Box::pin(async { Ok(()) }));

        mock_env
            .event_publisher
            .expect_user_tag_created()
            .withf(move |tag| {
                assert_eq!(tag.id, tag_id);
                assert_eq!(tag.owner_id, owner_id);
                assert_eq!(tag.name, "test");
                true
            })
            .return_once(|_| Box::pin(async { Ok(()) }));

        let cmd = CreateUserTag { owner_id, name };
        let tag = cmd.exec(&mock_env).await.unwrap();
        assert_eq!(tag.id, tag_id);
        assert_eq!(tag.owner_id, owner_id);
        assert_eq!(tag.name, "test");
    }

    #[tokio::test]
    #[rstest::rstest]
    async fn name_taken_error(mut mock_env: MockEnv) {
        let tag_id = TagId::generate();
        let owner_id = UserId::generate();
        let name = TagName::from("test");

        mock_env
            .id_generator
            .expect_generate_tag_id()
            .return_once(move || {
                let tag_id = tag_id;
                Box::pin(async move { Ok(tag_id) })
            });

        mock_env
            .meme_repo
            .expect_insert_user_tag()
            .withf(move |tag| {
                assert_eq!(tag.id, tag_id);
                assert_eq!(tag.owner_id, owner_id);
                assert_eq!(tag.name, "test");
                true
            })
            .return_once(|_| {
                Box::pin(async {
                    Err(InsertUserTagError::NameTaken {
                        name: TagName::from("test"),
                    })
                })
            });

        let cmd = CreateUserTag { owner_id, name };
        let err = cmd.exec(&mock_env).await.unwrap_err();
        assert!(matches!(
            err,
            CreateUserTagError::NameAlreadyExists { name } if name == "test"
        ));
    }
}
