use super::prelude::*;
use crate::entity::Meme;
use crate::types::{ByteSize, MemePath, RawFile};
use ::identity::UserId;

#[derive(::derive_more::Debug, Clone, PartialEq, Eq)]
pub struct CreateMeme {
    pub raw_file: RawFile,
    pub path: MemePath,
    pub owner_id: UserId,
}

impl<FM, MR, ID, EP> Command<FM, MR, ID, EP> for CreateMeme
where
    FM: FileManager,
    MR: MemeRepo,
    ID: IdGenerator,
    EP: EventPublisher,
{
    type Value = Meme;
    type Error = CreateMemeError;

    async fn exec(
        self,
        env: &Env<FM, MR, ID, EP>,
    ) -> Result<Self::Value, Self::Error> {
        let meme = Meme {
            id: env.id_generator.generate_meme_id().await?,
            owner_id: self.owner_id,
            path: self.path,
            file_size: ByteSize::b(self.raw_file.len() as u64),
        };

        let tasks = ::tokio::join!(
            env.file_manager.upload(&meme.path, &self.raw_file),
            env.meme_repo.insert(&meme),
        );

        let meme = match tasks {
            (Ok(()), Ok(())) => Ok(meme),
            (Err(upload), Err(insert)) => {
                Err(CreateMemeError::CompletelyFailed { upload, insert })
            }
            (Err(upload), Ok(())) => Err(CreateMemeError::UploadFailed {
                upload,
                insert_revert: env.meme_repo.delete(&meme.id).await.err(),
            }),
            (Ok(()), Err(insert)) => Err(CreateMemeError::InsertFailed {
                insert,
                upload_revert: env.file_manager.delete(&meme.path).await.err(),
            }),
        }?;

        env.event_publisher.meme_created(&meme).await?;

        Ok(meme)
    }
}

#[derive(Debug, ::thiserror::Error)]
pub enum CreateMemeError {
    #[error(transparent)]
    IdGenerattion(#[from] crate::port::id_generator::GenerateMemeIdError),

    #[error("CreateMeme upload and insert failed: {upload}, {insert}")]
    CompletelyFailed {
        upload: crate::port::file_manager::UploadError,
        insert: crate::port::meme_repo::InsertMemeError,
    },

    #[error("CreateMeme upload failed: {upload}, rever: {insert_revert:?}")]
    UploadFailed {
        upload: crate::port::file_manager::UploadError,
        insert_revert: Option<crate::port::meme_repo::DeleteByIdMemeError>,
    },

    #[error("CreateMeme insert failed: {insert}, revert: {upload_revert:?}")]
    InsertFailed {
        insert: crate::port::meme_repo::InsertMemeError,
        upload_revert: Option<crate::port::file_manager::DeleteError>,
    },

    #[error(transparent)]
    Event(#[from] crate::port::event_publisher::PublishError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::support::env::*;
    use crate::types::MemeId;

    #[tokio::test]
    #[rstest::rstest]
    async fn happy_path_works(mut mock_env: MockEnv) {
        let meme_id = MemeId::generate();
        let owner_id = UserId::generate();

        let cmd = CreateMeme {
            owner_id,
            raw_file: RawFile::from(vec![0, 1, 2]),
            path: MemePath::from("test-meme.jpg"),
        };

        mock_env
            .id_generator
            .expect_generate_meme_id()
            .return_once(move || {
                let meme_id = meme_id;
                Box::pin(async move { Ok(meme_id) })
            });

        mock_env
            .file_manager
            .expect_upload()
            .withf(|path, raw_file| {
                *path == "test-meme.jpg" && *raw_file == [0, 1, 2]
            })
            .return_once(|_, _| Box::pin(async { Ok(()) }));

        mock_env
            .meme_repo
            .expect_insert()
            .withf(move |meme| {
                meme.id == meme_id
                    && meme.owner_id == owner_id
                    && meme.path == "test-meme.jpg"
                    && meme.file_size == ByteSize::b(3)
            })
            .return_once(|_| Box::pin(async { Ok(()) }));

        mock_env
            .event_publisher
            .expect_meme_created()
            .withf(move |meme| {
                meme.id == meme_id
                    && meme.owner_id == owner_id
                    && meme.path == "test-meme.jpg"
                    && meme.file_size == ByteSize::b(3)
            })
            .return_once(|_| Box::pin(async { Ok(()) }));

        let meme = cmd.exec(&mock_env).await.expect("a meme to be created");

        assert_eq!(meme.id, meme_id);
        assert_eq!(meme.owner_id, owner_id);
        assert_eq!(meme.path, "test-meme.jpg");
        assert_eq!(meme.file_size, ByteSize::b(3));
    }
}
