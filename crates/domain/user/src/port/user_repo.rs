use crate::entity::{User, UserPasswordHash};
use crate::types::{UserId, Username};

#[derive(Debug, ::thiserror::Error)]
#[error("get user by id {id}: {source}")]
pub struct GetUserByIdError {
    pub id: UserId,
    pub source: Box<dyn ::std::error::Error + Send + Sync>,
}

#[derive(Debug, ::thiserror::Error)]
#[error("get by user name {username}: {source}")]
pub struct GetUserByNameError {
    pub username: Username,
    pub source: Box<dyn ::std::error::Error + Send + Sync>,
}

#[derive(Debug, ::thiserror::Error)]
pub enum InsertUserWithHashError {
    #[error("insert user with hash name taken: {username}")]
    NameTaken { username: Username },
    #[error("insert user with hash id taken: {id}")]
    IdTaken { id: UserId },
    #[error("insert user with hash unknown: {0}")]
    Unknown(#[from] ::anyhow::Error),
}

#[derive(Debug, ::thiserror::Error)]
#[error("get hash by user id {user_id}: {source}")]
pub struct GetHashByUserIdError {
    pub user_id: UserId,
    pub source: Box<dyn ::std::error::Error + Send + Sync>,
}

#[derive(Debug, ::thiserror::Error)]
#[error("get user and hash by name {uname}: {source}")]
pub struct GetUserAndHashByNameError {
    pub uname: String,
    pub source: Box<dyn ::std::error::Error + Send + Sync>,
}

#[derive(Debug, ::thiserror::Error)]
pub enum UpdateUserPasswordHashError {
    #[error("update user password hash user id not found: {id}")]
    UserNotFound { id: UserId },
    #[error("unknown: {0}")]
    Unknown(#[from] ::anyhow::Error),
}

#[derive(Debug, ::thiserror::Error)]
#[error(transparent)]
pub enum Error {
    GetUserById(#[from] GetUserByIdError),
    GetUserByName(#[from] GetUserByNameError),
    InsertUserWithHash(#[from] InsertUserWithHashError),
    GetUserAndHashByName(#[from] GetUserAndHashByNameError),
    UpdateUserPasswordHash(#[from] UpdateUserPasswordHashError),
    GehHashByUserId(#[from] GetHashByUserIdError),
}

#[cfg_attr(any(test, feature = "testing"), ::mockall::automock)]
pub trait UserRepo: Send + Sync {
    /// Returns the user with the given id, or None if no such user exists.
    fn get_user_by_id(
        &self,
        id: &UserId,
    ) -> impl Future<Output = Result<Option<User>, GetUserByIdError>> + Send;

    fn get_user_by_name(
        &self,
        uname: &str,
    ) -> impl Future<Output = Result<Option<User>, GetUserByNameError>> + Send;

    fn insert_user_with_hash(
        &self,
        user: &User,
        hash: &UserPasswordHash,
    ) -> impl Future<Output = Result<(), InsertUserWithHashError>> + Send;

    fn get_user_and_hash_by_name(
        &self,
        uname: &str,
    ) -> impl Future<
        Output = Result<Option<(User, UserPasswordHash)>, GetUserAndHashByNameError>,
    > + Send;

    fn update_user_password_hash(
        &self,
        hash: &UserPasswordHash,
    ) -> impl Future<Output = Result<(), UpdateUserPasswordHashError>> + Send;

    fn get_hash_by_user_id(
        &self,
        user_id: &UserId,
    ) -> impl Future<Output = Result<Option<UserPasswordHash>, GetHashByUserIdError>> + Send;
}

/// User Repository Tests
///
/// These tests are meant to be used by any implmentation of the
/// [user repo interface] to make sure that it behaves correctly.
/// This is not meant to be exhaustive, but it should cover the basic
/// functionality of the repo.  Please add more tests as needed when
/// any new functionality is added to the interface.
///
/// [user repo interface]: Interface
#[cfg(any(test, feature = "testing"))]
pub async fn test_adapter<R: UserRepo>(adapter: &R) -> Result<(), Error> {
    let user = User {
        id: UserId::generate(),
        username: "test".into(),
    };
    let hash = UserPasswordHash {
        user_id: user.id,
        hash: vec![1, 2, 3].into(),
    };

    adapter.insert_user_with_hash(&user, &hash).await?;

    let got_user = adapter
        .get_user_by_id(&user.id)
        .await?
        .expect("a user to be found by id");

    assert_eq!(got_user, user, "user found by id");

    let got_user = adapter
        .get_user_by_name(&user.username)
        .await?
        .expect("a user to be found by name");

    assert_eq!(got_user, user, "user found by username");

    let (got_user, got_hash) = adapter
        .get_user_and_hash_by_name(&user.username)
        .await?
        .expect("a user and hash by name");

    assert_eq!(got_user, user);
    assert_eq!(got_hash, hash);

    let already_named = User {
        id: UserId::generate(),
        username: user.username.clone(),
    };

    let err = adapter
        .insert_user_with_hash(&already_named, &hash)
        .await
        .expect_err("inserting a user with a taken name should fail");

    assert!(
        matches!(err, InsertUserWithHashError::NameTaken { username } if username == user.username),
        "inserting a user with a taken name should fail with the correct name"
    );

    let already_id = User {
        id: user.id,
        username: "other".into(),
    };

    let err = adapter
        .insert_user_with_hash(&already_id, &hash)
        .await
        .expect_err("inserting a user with a taken id should fail");

    assert!(
        matches!(err, InsertUserWithHashError::IdTaken { id } if id == user.id),
        "inserting a user with a taken id should fail with the correct id"
    );

    let fetched_hash = adapter
        .get_hash_by_user_id(&user.id)
        .await?
        .expect("a hash by user id");

    assert_eq!(
        &fetched_hash, &hash,
        "expected hash by user id after insert"
    );

    let updated_hash = UserPasswordHash {
        user_id: user.id,
        hash: vec![4, 5, 6].into(),
    };

    adapter.update_user_password_hash(&updated_hash).await?;

    let (got_user, got_hash) = adapter
        .get_user_and_hash_by_name(&user.username)
        .await?
        .expect("a user and hash by name");

    assert_eq!(got_user, user);
    assert_eq!(got_hash, updated_hash);

    let bad_hash = UserPasswordHash {
        user_id: UserId::generate(),
        hash: vec![7, 8, 9].into(),
    };

    let err = adapter
        .update_user_password_hash(&bad_hash)
        .await
        .expect_err("updating a non-existent user's password hash should fail");

    assert!(
        matches!(err, UpdateUserPasswordHashError::UserNotFound { id } if id == bad_hash.user_id),
        "updating a non-existent user's password hash should fail with the correct user id"
    );

    let (got_user, got_hash) = adapter
        .get_user_and_hash_by_name(&user.username)
        .await?
        .expect("a user and hash by name");

    assert_eq!(got_user, user);
    assert_eq!(got_hash, updated_hash);

    Ok(())
}
