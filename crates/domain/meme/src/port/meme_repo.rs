use crate::entity::Meme;
use crate::types::{MemeId, MemePath};

#[cfg_attr(any(test, feature = "testing"), ::mockall::automock)]
pub trait MemeRepo: Send + Sync {
    fn insert(
        &self,
        meme: &Meme,
    ) -> impl Future<Output = Result<(), InsertMemeError>> + Send;

    fn delete(
        &self,
        id: &MemeId,
    ) -> impl Future<Output = Result<(), DeleteByIdMemeError>> + Send;

    fn update_by_id(
        &self,
        meme: &Meme,
    ) -> impl Future<Output = Result<(), UpdateByIdMemeError>> + Send;

    fn fetch_by_id(
        &self,
        id: &MemeId,
    ) -> impl Future<Output = Result<Meme, FetchByIdError>> + Send;
}

#[derive(Debug, ::thiserror::Error)]
#[error(transparent)]
pub enum MemeRepoError {
    Insert(#[from] InsertMemeError),
    DeleteById(#[from] DeleteByIdMemeError),
    UpdateById(#[from] UpdateByIdMemeError),
    FetchById(#[from] FetchByIdError),
}

#[derive(Debug, ::thiserror::Error)]
pub enum InsertMemeError {
    #[error("insert-meme already-exists: {id}")]
    MemeAlreadyExists { id: MemeId },
    #[error("insert-meme path-taken: {path}")]
    PathTaken { path: MemePath },
    #[error(transparent)]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, ::thiserror::Error)]
pub enum DeleteByIdMemeError {
    #[error("update-by-id not-found: {id}")]
    MemeNotFound { id: MemeId },
    #[error(transparent)]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, ::thiserror::Error)]
pub enum UpdateByIdMemeError {
    #[error("update-by-id not-found: {id}")]
    MemeNotFound { id: MemeId },
    #[error("update-by-id path-taken: {path}")]
    PathTaken { path: MemePath },
    #[error(transparent)]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Debug, ::thiserror::Error)]
pub enum FetchByIdError {
    #[error("fetch-by-id not-found: {id}")]
    MemeNotFound { id: MemeId },
    #[error(transparent)]
    Unknown(Box<dyn std::error::Error + Send + Sync>),
}

#[cfg(any(test, feature = "testing"))]
pub async fn test_meme_repo<T: MemeRepo>(repo: &T) -> Result<(), MemeRepoError> {
    use ::fake::{Fake, Faker};

    let meme = Faker.fake::<Meme>();

    // Happy path insert, fetch, delete, not-found
    repo.insert(&meme).await?;
    assert_eq!(repo.fetch_by_id(&meme.id).await?, meme);
    repo.delete(&meme.id).await?;
    let Err(missing) = repo.fetch_by_id(&meme.id).await else {
        panic!("expected fetch-by-id to fail after delete");
    };
    assert!(matches!(
        missing,
        FetchByIdError::MemeNotFound { id } if id == meme.id
    ));

    // Attempt to insert with duplicate path
    let mut another = Faker.fake::<Meme>();
    another.path = meme.path.clone();
    repo.insert(&meme).await?;
    let Err(error) = repo.insert(&another).await else {
        panic!("expected insert to fail when path is taken");
    };
    assert!(matches!(
        error,
        InsertMemeError::PathTaken { path } if path == another.path
    ));
    repo.delete(&meme.id).await?;

    // Happy path update-by-id, fetch, delete
    let mut updated = Faker.fake::<Meme>();
    updated.id = meme.id;
    repo.insert(&meme).await?;
    repo.update_by_id(&updated).await?;
    assert_eq!(repo.fetch_by_id(&meme.id).await?, updated);
    repo.delete(&meme.id).await?;

    // Attempt to update with duplicate path
    let mut another = Faker.fake::<Meme>();
    repo.insert(&meme).await?;
    repo.insert(&another).await?;
    another.path = meme.path.clone();
    let Err(error) = repo.update_by_id(&another).await else {
        panic!("expected update-by-id to fail when path is taken");
    };
    assert!(matches!(
        error,
        UpdateByIdMemeError::PathTaken { path } if path == another.path
    ));
    repo.delete(&meme.id).await?;
    repo.delete(&another.id).await?;

    Ok(())
}
