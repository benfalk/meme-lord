use super::prelude::*;
use crate::entity::User;
use crate::types::UserId;

#[derive(Debug, Clone)]
pub struct FindById {
    pub id: UserId,
}

#[derive(Debug, ::thiserror::Error)]
pub enum FindByIdError {
    #[error(transparent)]
    RepoError(crate::port::user_repo::GetUserByIdError),
}

impl Query for FindById {
    type Value = Option<User>;
    type Error = FindByIdError;

    async fn query(self, env: &impl EnvExt) -> Result<Self::Value, Self::Error> {
        env.user_repo()
            .get_user_by_id(&self.id)
            .await
            .map_err(FindByIdError::RepoError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::support::env::*;
    use crate::types::UserId;
    use ::fake::{Fake, Faker};

    #[tokio::test]
    #[rstest::rstest]
    async fn happy_path(mut mock_env: MockEnv) {
        let user: User = Faker.fake();
        let user_id = user.id;
        let find_by_id = FindById { id: user.id };
        let returned_user = user.clone();

        mock_env
            .user_repo
            .expect_get_user_by_id()
            .withf(move |id| *id == user_id)
            .returning(move |_| {
                let value = returned_user.clone();
                Box::pin(async move { Ok(Some(value)) })
            });

        let Some(found_user) = find_by_id.query(&mock_env).await.unwrap() else {
            panic!("Expected to find a user");
        };

        assert_eq!(found_user, user);
    }

    #[tokio::test]
    #[rstest::rstest]
    async fn when_repo_fails_error_is_returned_user_id(mut mock_env: MockEnv) {
        let user_id = UserId::generate();
        let find_by_id = FindById { id: user_id };

        mock_env
            .user_repo
            .expect_get_user_by_id()
            .withf(move |id| *id == user_id)
            .returning(move |_| {
                Box::pin(async move {
                    Err(crate::port::user_repo::GetUserByIdError {
                        id: user_id,
                        source: "Database error".into(),
                    })
                })
            });

        let err = find_by_id.query(&mock_env).await.unwrap_err();

        assert!(matches!(
            err,
            FindByIdError::RepoError(crate::port::user_repo::GetUserByIdError { id, source })
            if id == user_id && source.to_string() == "Database error"
        ));
    }
}
