use super::prelude::*;
use crate::entity::UserPasswordHash;
use crate::service::EnvExt;
use crate::types::{Password, UserId};

#[derive(Debug)]
pub struct ChangePassword {
    pub user_id: UserId,
    pub new_password: Password,
    pub current_password: Password,
}

#[derive(Debug, ::thiserror::Error)]
pub enum ChangePasswordError {
    #[error("invalid current password")]
    InvalidCurrentPassword,

    #[error("invalid userid: {0}")]
    InvalidUserId(crate::types::UserId),

    #[error(transparent)]
    Hash(#[from] crate::port::password_hasher::HashError),

    #[error(transparent)]
    Repo(crate::port::user_repo::Error),

    #[error(transparent)]
    Publish(#[from] crate::port::event_publisher::PublishError),
}

impl Command for ChangePassword {
    type Error = ChangePasswordError;
    type Value = ();

    async fn exec(self, env: &impl EnvExt) -> Result<Self::Value, Self::Error> {
        let db_hash = env
            .user_repo()
            .get_hash_by_user_id(&self.user_id)
            .await?
            .ok_or_else(|| ChangePasswordError::InvalidUserId(self.user_id))?;

        env.password_hasher()
            .verify(&self.current_password, &db_hash.hash)
            .await?
            .then_some(())
            .ok_or(ChangePasswordError::InvalidCurrentPassword)?;

        let new_hash = env.password_hasher().hash(&self.new_password).await?;

        env.user_repo()
            .update_user_password_hash(&UserPasswordHash {
                user_id: self.user_id,
                hash: new_hash,
            })
            .await?;

        env.event_publisher()
            .user_password_changed(self.user_id)
            .await?;

        Ok(())
    }
}

mod impls {
    use super::*;
    use crate::port::user_repo;

    impl<T: Into<user_repo::Error>> From<T> for ChangePasswordError {
        fn from(value: T) -> Self {
            Self::Repo(value.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::support::env::*;

    #[tokio::test]
    #[rstest::rstest]
    async fn happy_path(mut mock_env: MockEnv) {
        let user_id = UserId::generate();
        let current_password = Password::from("current-password");
        let new_password = Password::from("new-password");

        mock_env
            .user_repo
            .expect_get_hash_by_user_id()
            .withf(move |id| *id == user_id)
            .returning(move |_| {
                let hash = vec![1, 2, 3].into();
                Box::pin(async move { Ok(Some(UserPasswordHash { user_id, hash })) })
            });

        mock_env
            .password_hasher
            .expect_verify()
            .withf(move |password, hash| {
                assert_eq!(password.as_ref(), "current-password".as_bytes());
                assert_eq!(hash.as_bytes(), &[1, 2, 3]);
                true
            })
            .returning(|_, _| Box::pin(async move { Ok(true) }));

        mock_env
            .password_hasher
            .expect_hash()
            .withf(move |password| {
                assert_eq!(password.as_ref(), "new-password".as_bytes());
                true
            })
            .returning(|_| Box::pin(async move { Ok(vec![4, 5, 6].into()) }));

        mock_env
            .user_repo
            .expect_update_user_password_hash()
            .withf(move |hash| {
                hash.user_id == user_id && hash.hash == vec![4, 5, 6].into()
            })
            .returning(|_| Box::pin(async { Ok(()) }));

        mock_env
            .event_publisher
            .expect_user_password_changed()
            .withf(move |given_user_id| *given_user_id == user_id)
            .returning(|_| Box::pin(async { Ok(()) }));

        let change_password = ChangePassword {
            user_id,
            current_password,
            new_password,
        };

        change_password.exec(&mock_env).await.unwrap()
    }

    #[tokio::test]
    #[rstest::rstest]
    async fn when_user_not_found_then_error(mut mock_env: MockEnv) {
        let user_id = UserId::generate();

        mock_env
            .user_repo
            .expect_get_hash_by_user_id()
            .withf(move |id| *id == user_id)
            .returning(move |_| Box::pin(async move { Ok(None) }));

        let change_password = ChangePassword {
            user_id,
            current_password: Password::from("any-password"),
            new_password: Password::from("any-new-password"),
        };

        let err = change_password.exec(&mock_env).await.unwrap_err();
        assert!(
            matches!(err, ChangePasswordError::InvalidUserId(id) if id == user_id),
            "error should be InvalidUserId with the correct id"
        );
    }

    #[tokio::test]
    #[rstest::rstest]
    async fn when_current_password_invalid_then_error(mut mock_env: MockEnv) {
        let user_id = UserId::generate();

        mock_env
            .user_repo
            .expect_get_hash_by_user_id()
            .withf(move |id| *id == user_id)
            .returning(move |_| {
                let hash = vec![1, 2, 3].into();
                Box::pin(async move { Ok(Some(UserPasswordHash { user_id, hash })) })
            });

        mock_env
            .password_hasher
            .expect_verify()
            .withf(move |password, hash| {
                assert_eq!(password.as_ref(), "wrong-current-password".as_bytes());
                assert_eq!(hash.as_bytes(), &[1, 2, 3]);
                true
            })
            .returning(|_, _| Box::pin(async move { Ok(false) }));

        let change_password = ChangePassword {
            user_id,
            current_password: Password::from("wrong-current-password"),
            new_password: Password::from("any-new-password"),
        };

        let err = change_password.exec(&mock_env).await.unwrap_err();
        assert!(
            matches!(err, ChangePasswordError::InvalidCurrentPassword),
            "error should be InvalidCurrentPassword"
        );
    }
}
