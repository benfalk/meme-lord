use super::prelude::*;
use crate::entity::User;
use crate::types::Username;

#[derive(Debug, Clone)]
pub struct FindByUsername {
    pub username: Username,
}

#[derive(Debug, ::thiserror::Error)]
pub enum FindByUsernameError {
    #[error(transparent)]
    RepoError(crate::port::user_repo::GetUserByNameError),
}

impl<UR, PH, EP> Query<UR, PH, EP> for FindByUsername
where
    UR: UserRepo,
    PH: PasswordHasher,
    EP: EventPublisher,
{
    type Value = Option<User>;
    type Error = FindByUsernameError;

    async fn query(self, env: &Env<UR, PH, EP>) -> Result<Self::Value, Self::Error> {
        env.user_repo
            .get_user_by_name(&self.username)
            .await
            .map_err(FindByUsernameError::RepoError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::support::env::*;
    use crate::types::Username;
    use ::fake::{Fake, Faker};

    #[tokio::test]
    #[rstest::rstest]
    async fn happy_path(mut mock_env: MockEnv) {
        let user: User = Faker.fake();
        let username = user.username.clone();
        let find_by_username = FindByUsername {
            username: user.username.clone(),
        };
        let returned_user = user.clone();

        mock_env
            .user_repo
            .expect_get_user_by_name()
            .withf(move |name| username == name)
            .returning(move |_| {
                let value = returned_user.clone();
                Box::pin(async move { Ok(Some(value)) })
            });

        let Some(found_user) = find_by_username.query(&mock_env).await.unwrap()
        else {
            panic!("Expected to find a user");
        };

        assert_eq!(found_user, user);
    }

    #[tokio::test]
    #[rstest::rstest]
    async fn when_repo_fails_error_is_returned_with_username(mut mock_env: MockEnv) {
        let username = Username::new("test-user");
        let find_by_username = FindByUsername {
            username: username.clone(),
        };

        mock_env
            .user_repo
            .expect_get_user_by_name()
            .withf(move |name| name == "test-user")
            .returning(move |_| {
                let username = username.clone();
                Box::pin(async move {
                    let username = username.clone();
                    Err(crate::port::user_repo::GetUserByNameError {
                        username,
                        source: "oh no".into(),
                    })
                })
            });

        let err = find_by_username.query(&mock_env).await.unwrap_err();

        assert!(matches!(
            err,
            FindByUsernameError::RepoError(crate::port::user_repo::GetUserByNameError {
                username,
                source
            }) if username == "test-user" && source.to_string() == "oh no"
        ));
    }
}
