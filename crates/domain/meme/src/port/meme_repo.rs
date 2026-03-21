use crate::entity::Meme;
use crate::types::{MemeId, MemePath};

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
