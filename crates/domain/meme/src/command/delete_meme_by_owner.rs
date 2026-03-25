use super::prelude::*;
use crate::types::MemeId;
use ::identity::UserId;

#[derive(::derive_more::Debug, Clone, PartialEq, Eq)]
pub struct DeleteMemeByOwner {
    pub meme_id: MemeId,
    pub owner_id: UserId,
}

impl<FM, MR, ID, EP> Command<FM, MR, ID, EP> for DeleteMemeByOwner
where
    FM: FileManager,
    MR: MemeRepo,
    ID: IdGenerator,
    EP: EventPublisher,
{
    type Value = ();
    type Error = DeleteMemeByOwnerError;

    async fn exec(
        self,
        env: &Env<FM, MR, ID, EP>,
    ) -> Result<Self::Value, Self::Error> {
        let meme = env.meme_repo.fetch_by_id(&self.meme_id).await.map_err(
            |err| match err {
                crate::port::meme_repo::FetchByIdError::MemeNotFound { id } => {
                    DeleteMemeByOwnerError::NotFound { meme_id: id }
                }
                err => DeleteMemeByOwnerError::FetchById(err),
            },
        )?;

        if meme.owner_id != self.owner_id {
            return Err(DeleteMemeByOwnerError::PermissionDenied {
                meme_id: self.meme_id,
                owner_id: self.owner_id,
            });
        }

        let tasks = ::tokio::join!(
            env.file_manager.delete(&meme.path),
            env.meme_repo.delete(&self.meme_id),
        );

        match tasks {
            (Ok(_), Ok(())) => Ok(()),
            (Err(file_manager), Err(repo)) => {
                Err(DeleteMemeByOwnerError::CompleteFailure { file_manager, repo })
            }
            (Err(file_manager), Ok(())) => {
                Err(DeleteMemeByOwnerError::FileManagerDelete(file_manager))
            }
            (Ok(_), Err(repo)) => Err(DeleteMemeByOwnerError::RepoDelete(repo)),
        }?;

        env.event_publisher.meme_deleted(meme.id).await?;

        Ok(())
    }
}

#[derive(Debug, ::thiserror::Error)]
pub enum DeleteMemeByOwnerError {
    #[error("meme not found: {meme_id}")]
    NotFound { meme_id: MemeId },
    #[error("complete failure: {file_manager} | {repo}")]
    CompleteFailure {
        file_manager: crate::port::file_manager::DeleteError,
        repo: crate::port::meme_repo::DeleteByIdMemeError,
    },
    #[error("cannot delete {meme_id} for owner {owner_id}")]
    PermissionDenied { meme_id: MemeId, owner_id: UserId },
    #[error(transparent)]
    FileManagerDelete(#[from] crate::port::file_manager::DeleteError),
    #[error(transparent)]
    RepoDelete(#[from] crate::port::meme_repo::DeleteByIdMemeError),
    #[error(transparent)]
    FetchById(#[from] crate::port::meme_repo::FetchByIdError),
    #[error(transparent)]
    Event(#[from] crate::port::event_publisher::PublishError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::Meme;
    use crate::support::env::*;
    use crate::types::ByteSize;

    #[tokio::test]
    #[rstest::rstest]
    async fn happy_path_works(mut mock_env: MockEnv) {
        let meme_id = MemeId::generate();
        let owner_id = UserId::generate();
        let cmd = DeleteMemeByOwner { meme_id, owner_id };

        mock_env
            .meme_repo
            .expect_fetch_by_id()
            .withf(move |id| *id == meme_id)
            .returning(move |_| {
                let meme_id = meme_id;
                let owner_id = owner_id;
                Box::pin(async move {
                    Ok(Meme {
                        id: meme_id,
                        owner_id,
                        path: "test.jpg".into(),
                        file_size: ByteSize(123),
                    })
                })
            });

        mock_env
            .file_manager
            .expect_delete()
            .withf(|path| *path == "test.jpg")
            .return_once(|_| {
                Box::pin(async {
                    Ok(crate::port::file_manager::DeleteStatus::Deleted)
                })
            });

        mock_env
            .meme_repo
            .expect_delete()
            .withf(move |id| *id == meme_id)
            .return_once(|_| Box::pin(async { Ok(()) }));

        mock_env
            .event_publisher
            .expect_meme_deleted()
            .withf(move |id| *id == meme_id)
            .return_once(|_| Box::pin(async { Ok(()) }));

        cmd.exec(&mock_env).await.expect("deletion to succeed");
    }
}
