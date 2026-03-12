use super::prelude::*;
use crate::entity::User;
use crate::types::{Password, Username};

#[derive(Debug)]
pub struct SignIn {
    pub username: Username,
    pub password: Password,
}

#[derive(Debug, ::thiserror::Error)]
pub enum SignInError {
    #[error("invalid username or password")]
    InvalidUsernameOrPassword,

    #[error(transparent)]
    Hash(#[from] crate::port::password_hasher::HashError),

    #[error(transparent)]
    RepoError(crate::port::user_repo::GetUserAndHashByNameError),
}

impl<UR, PH> Command<UR, PH> for SignIn
where
    UR: UserRepo,
    PH: PasswordHasher,
{
    type Error = SignInError;
    type Value = User;

    async fn exec(self, env: &Env<UR, PH>) -> Result<Self::Value, Self::Error> {
        let (user, password) = env
            .user_repo
            .get_user_and_hash_by_name(&self.username)
            .await
            .map_err(SignInError::RepoError)?
            .ok_or(SignInError::InvalidUsernameOrPassword)?;

        env.password_hasher
            .verify(&self.password, &password.hash)
            .await
            .map_err(SignInError::Hash)?
            .then_some(user)
            .ok_or(SignInError::InvalidUsernameOrPassword)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::UserPasswordHash;
    use crate::support::env::*;
    use crate::types::UserId;

    #[rstest::rstest]
    #[tokio::test]
    async fn happy_path(mut mock_env: MockEnv) {
        let username = Username::new("test-user");
        let sign_in = SignIn {
            username: username.clone(),
            password: Password::from("test-password"),
        };
        let user = User {
            id: UserId::generate(),
            username: username.clone(),
        };
        let hash = UserPasswordHash {
            user_id: user.id,
            hash: vec![1, 2, 3].into(),
        };
        let user_and_hash = (user.clone(), hash.clone());

        mock_env
            .user_repo
            .expect_get_user_and_hash_by_name()
            .withf(move |name| username == name)
            .returning(move |_| {
                let value = user_and_hash.clone();
                Box::pin(async move { Ok(Some(value)) })
            });

        mock_env
            .password_hasher
            .expect_verify()
            .withf(move |password, hash| {
                assert_eq!(password.as_ref(), "test-password".as_bytes());
                assert_eq!(hash.as_bytes(), &[1, 2, 3]);
                true
            })
            .returning(|_, _| Box::pin(async { Ok(true) }));

        let returned_user = sign_in.exec(&mock_env).await.unwrap();
        assert_eq!(returned_user, user);
    }

    #[rstest::rstest]
    #[tokio::test]
    async fn when_user_and_hash_not_found_then_error(mut mock_env: MockEnv) {
        let sign_in = SignIn {
            username: Username::from("nonexistent-user"),
            password: Password::from("any-password"),
        };

        mock_env
            .user_repo
            .expect_get_user_and_hash_by_name()
            .withf(|name| name == "nonexistent-user")
            .returning(|_| Box::pin(async { Ok(None) }));

        assert!(matches!(
            sign_in.exec(&mock_env).await.unwrap_err(),
            SignInError::InvalidUsernameOrPassword
        ));
    }

    #[rstest::rstest]
    #[tokio::test]
    async fn when_password_does_not_verify_then_error(mut mock_env: MockEnv) {
        let sign_in = SignIn {
            username: Username::from("test-user"),
            password: Password::from("wrong-password"),
        };
        let user = User {
            id: UserId::generate(),
            username: sign_in.username.clone(),
        };
        let hash = UserPasswordHash {
            user_id: user.id,
            hash: vec![1, 2, 3].into(),
        };
        let user_and_hash = (user.clone(), hash.clone());

        mock_env
            .user_repo
            .expect_get_user_and_hash_by_name()
            .withf(move |name| name == "test-user")
            .returning(move |_| {
                let value = user_and_hash.clone();
                Box::pin(async move { Ok(Some(value)) })
            });

        mock_env
            .password_hasher
            .expect_verify()
            .withf(|password, hash| {
                assert_eq!(password.as_ref(), "wrong-password".as_bytes());
                assert_eq!(hash.as_bytes(), &[1, 2, 3]);
                true
            })
            .returning(|_, _| Box::pin(async { Ok(false) }));

        assert!(matches!(
            sign_in.exec(&mock_env).await.unwrap_err(),
            SignInError::InvalidUsernameOrPassword
        ));
    }
}
