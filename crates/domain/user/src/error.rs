#[derive(Debug, ::thiserror::Error)]
pub enum Error {
    #[error("user repo -> {0}")]
    UserRepo(crate::port::user_repo::Error),

    #[error("user id -> {0}")]
    UserId(#[from] crate::types::UserIdError),

    #[error("password hash -> {0}")]
    PasswordHash(#[from] crate::port::password_hasher::HashError),
}

impl<T: Into<crate::port::user_repo::Error>> From<T> for Error {
    fn from(value: T) -> Self {
        Self::UserRepo(value.into())
    }
}

impl From<crate::types::UserIdParseError> for Error {
    fn from(value: crate::types::UserIdParseError) -> Self {
        Self::UserId(value.into())
    }
}

impl From<crate::types::UserIdVersionError> for Error {
    fn from(value: crate::types::UserIdVersionError) -> Self {
        Self::UserId(value.into())
    }
}
