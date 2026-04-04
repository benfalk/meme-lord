use super::prelude::*;
use crate::entity::UserTag;
use ::identity::UserId;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FetchUserTags {
    pub user_id: UserId,
}

impl Query for FetchUserTags {
    type Value = Vec<UserTag>;
    type Error = FetchError;

    async fn query(self, env: &impl EnvExt) -> Result<Self::Value, Self::Error> {
        env.meme_repo()
            .user_tags(&self.user_id)
            .await
            .map_err(|e| FetchError::Unknown(Box::new(e)))
    }
}

#[derive(Debug, ::thiserror::Error)]
pub enum FetchError {
    #[error(transparent)]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::support::env::*;
    use ::fake::{Fake, Faker};

    #[tokio::test]
    #[rstest::rstest]
    async fn happy_path_works(mut mock_env: MockEnv) {
        let user_id = UserId::generate();
        let expected_tags = ::std::iter::repeat_with(|| {
            let mut tag = Faker.fake::<UserTag>();
            tag.owner_id = user_id;
            tag
        })
        .take(3)
        .collect::<Vec<_>>();

        mock_env
            .meme_repo
            .expect_user_tags()
            .withf(move |id| *id == user_id)
            .returning({
                let for_return = expected_tags.clone();
                move |_| {
                    let tags = for_return.clone();
                    Box::pin(async move { Ok(tags) })
                }
            });

        let query = FetchUserTags { user_id };
        let tags = query.query(&mock_env).await.unwrap();
        assert_eq!(tags, expected_tags);
    }
}
