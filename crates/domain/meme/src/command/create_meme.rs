use super::prelude::*;
use crate::entity::Meme;
use crate::types::{ByteSize, MemeId, MemePath, RawFile};
use ::identity::UserId;

#[derive(::derive_more::Debug, Clone, PartialEq, Eq)]
pub struct CreateMeme {
    pub raw_file: RawFile,
    pub path: MemePath,
    pub meme_id: MemeId,
    pub owner_id: UserId,
}

impl<FM, MR> Command<FM, MR> for CreateMeme
where
    FM: FileManager,
    MR: MemeRepo,
{
    type Value = Meme;
    type Error = CreateMemeError;

    async fn exec(self, env: &Env<FM, MR>) -> Result<Self::Value, Self::Error> {
        let meme = Meme {
            id: self.meme_id,
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

        // Right here is where we can publish an event if we want to

        Ok(meme)
    }
}

#[derive(Debug, ::thiserror::Error)]
pub enum CreateMemeError {
    #[error("CreateMeme upload and insert failed: {upload}, {insert}")]
    CompletelyFailed {
        upload: crate::port::file_uploader::UploadError,
        insert: crate::port::meme_repo::InsertMemeError,
    },

    #[error("CreateMeme upload failed: {upload}, rever: {insert_revert:?}")]
    UploadFailed {
        upload: crate::port::file_uploader::UploadError,
        insert_revert: Option<crate::port::meme_repo::DeleteByIdMemeError>,
    },

    #[error("CreateMeme insert failed: {insert}, revert: {upload_revert:?}")]
    InsertFailed {
        insert: crate::port::meme_repo::InsertMemeError,
        upload_revert: Option<crate::port::file_uploader::DeleteError>,
    },
}
