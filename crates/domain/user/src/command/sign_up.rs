use super::prelude::*;
use crate::entity::{User, UserPasswordHash};
use crate::types::{Password, UserId, Username};

#[derive(Debug)]
pub struct SignUp {
    pub username: Username,
    pub password: Password,
}

#[derive(Debug, ::thiserror::Error)]
pub enum SignUpError {
    #[error("username taken: {username}")]
    UsernameTaken { username: Username },

    #[error(transparent)]
    Hash(#[from] crate::port::password_hasher::HashError),

    #[error(transparent)]
    Insert(crate::port::user_repo::InsertUserWithHashError),

    #[error(transparent)]
    Publish(#[from] crate::port::event_publisher::PublishError),
}

impl Command for SignUp {
    type Error = SignUpError;
    type Value = User;

    async fn exec(self, env: &impl EnvExt) -> Result<Self::Value, Self::Error> {
        let hash = env.password_hasher().hash(&self.password).await?;
        let user_id = UserId::generate();
        let user = User {
            id: user_id,
            username: self.username,
        };
        let hash = UserPasswordHash { user_id, hash };
        env.user_repo().insert_user_with_hash(&user, &hash).await?;
        env.event_publisher().user_created(user.id).await?;
        Ok(user)
    }
}

mod impls {
    use super::*;
    use crate::port::user_repo::InsertUserWithHashError;

    impl From<InsertUserWithHashError> for SignUpError {
        fn from(value: InsertUserWithHashError) -> Self {
            match value {
                InsertUserWithHashError::NameTaken { username } => {
                    Self::UsernameTaken { username }
                }
                other => Self::Insert(other),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::port::user_repo::InsertUserWithHashError;
    use crate::support::env::*;

    #[rstest::rstest]
    #[tokio::test]
    async fn sign_up_happy_path(mut mock_env: MockEnv) {
        let sign_up = SignUp {
            username: "new_user".into(),
            password: "my_secret_password".into(),
        };

        mock_env
            .password_hasher
            .expect_hash()
            .withf(|password| password.as_ref() == "my_secret_password".as_bytes())
            .returning(|_| Box::pin(async { Ok(vec![1, 2, 3].into()) }));

        mock_env
            .user_repo
            .expect_insert_user_with_hash()
            .withf(|user, password| {
                assert_eq!(user.username, "new_user");
                assert_eq!(password.hash.as_bytes(), &[1, 2, 3]);
                assert_eq!(user.id, password.user_id);
                true
            })
            .returning(|_, _| Box::pin(async move { Ok(()) }));

        mock_env
            .event_publisher
            .expect_user_created()
            .returning(|_| Box::pin(async { Ok(()) }));

        let user = sign_up.exec(&mock_env).await.unwrap();
        assert_eq!(user.username, "new_user");
    }

    #[rstest::rstest]
    #[tokio::test]
    async fn when_username_taken_then_error(mut mock_env: MockEnv) {
        let cmd = SignUp {
            username: "existing_user".into(),
            password: "my_secret_password".into(),
        };

        mock_env
            .password_hasher
            .expect_hash()
            .returning(|_| Box::pin(async { Ok(vec![1, 2, 3].into()) }));

        mock_env
            .user_repo
            .expect_insert_user_with_hash()
            .returning(|_, _| {
                Box::pin(async move {
                    Err(InsertUserWithHashError::NameTaken {
                        username: Username::from("existing_user"),
                    })
                })
            });

        let err = cmd.exec(&mock_env).await.unwrap_err();

        match err {
            SignUpError::UsernameTaken { username } => {
                assert_eq!(username, "existing_user");
            }
            other => panic!("unexpected error: {other}"),
        }
    }
}
