use super::prelude::*;
use crate::entity::Meme;
use crate::port::meme_repo::FetchMemeByIdError;
use crate::types::MemeId;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FetchMemeById {
    pub meme_id: MemeId,
}

impl Query for FetchMemeById {
    type Value = Meme;
    type Error = FetchError;

    async fn query(self, env: &impl EnvExt) -> Result<Self::Value, Self::Error> {
        env.meme_repo()
            .fetch_meme_by_id(&self.meme_id)
            .await
            .map_err(|e| match e {
                FetchMemeByIdError::MemeNotFound { id } => {
                    FetchError::MemeNotFound { id }
                }
                FetchMemeByIdError::Unknown(e) => FetchError::Unknown(e),
            })
    }
}

#[derive(Debug, ::thiserror::Error)]
pub enum FetchError {
    #[error("not found: {id}")]
    MemeNotFound { id: MemeId },
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
        let meme = Faker.fake::<Meme>();
        let meme_id = meme.id;
        let fetch_meme_by_id = FetchMemeById { meme_id };

        mock_env
            .meme_repo
            .expect_fetch_meme_by_id()
            .withf(move |id| id == &meme_id)
            .returning({
                let for_return = meme.clone();
                move |_| {
                    let meme = for_return.clone();
                    Box::pin(async move { Ok(meme) })
                }
            });

        let found = fetch_meme_by_id
            .query(&mock_env)
            .await
            .expect("a meme to be found");

        assert_eq!(meme, found);
    }
}
