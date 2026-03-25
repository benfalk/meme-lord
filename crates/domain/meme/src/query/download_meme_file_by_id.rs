use super::prelude::*;
use crate::port::file_manager::DownloadError;
use crate::port::meme_repo::FetchByIdError;
use crate::types::{MemeId, RawFile};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DownloadMemeFileById {
    pub meme_id: MemeId,
}

impl<FM, MR, ID, EP> Query<FM, MR, ID, EP> for DownloadMemeFileById
where
    FM: FileManager,
    MR: MemeRepo,
    ID: IdGenerator,
    EP: EventPublisher,
{
    type Value = RawFile;
    type Error = DownloadMemeFileByIdError;

    async fn query(
        self,
        env: &Env<FM, MR, ID, EP>,
    ) -> Result<Self::Value, Self::Error> {
        let meme = env
            .meme_repo
            .fetch_by_id(&self.meme_id)
            .await
            .map_err(|e| match e {
                FetchByIdError::MemeNotFound { id } => {
                    DownloadMemeFileByIdError::MemeNotFound { id }
                }
                FetchByIdError::Unknown(e) => DownloadMemeFileByIdError::Unknown(e),
            })?;

        env.file_manager
            .download(&meme.path)
            .await
            .map_err(|e| match e {
                DownloadError::FileNotFound { .. } => {
                    // This should not happen if the repo and file manager
                    // are consistent, but we can treat it as a not found
                    // error for simplicity
                    DownloadMemeFileByIdError::MemeNotFound { id: self.meme_id }
                }
                unknown => {
                    DownloadMemeFileByIdError::Unknown(Box::new(unknown)
                        as Box<dyn std::error::Error + Send + Sync>)
                }
            })
    }
}

#[derive(Debug, ::thiserror::Error)]
pub enum DownloadMemeFileByIdError {
    #[error("not found: {id}")]
    MemeNotFound { id: MemeId },
    #[error(transparent)]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{entity::Meme, support::env::*};
    use ::fake::{Fake, Faker};

    #[tokio::test]
    #[rstest::rstest]
    async fn happy_path(mut mock_env: MockEnv) {
        let meme = Faker.fake::<Meme>();
        let meme_id = meme.id;
        let expected_path = meme.path.clone();
        let download_by_id = DownloadMemeFileById { meme_id };

        mock_env
            .meme_repo
            .expect_fetch_by_id()
            .withf(move |id| id == &meme_id)
            .returning(move |_| {
                let meme = meme.clone();
                Box::pin(async move { Ok(meme) })
            });

        mock_env
            .file_manager
            .expect_download()
            .withf(move |path| path == &expected_path)
            .returning(|_| Box::pin(async { Ok(RawFile::from([1, 2, 3])) }));

        let data = download_by_id
            .query(&mock_env)
            .await
            .expect("to find file data");

        assert_eq!(data, RawFile::from([1, 2, 3]));
    }
}
