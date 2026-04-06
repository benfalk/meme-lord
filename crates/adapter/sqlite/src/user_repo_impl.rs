use crate::Sqlite;
use ::user::entity::{User, UserPasswordHash};
use ::user::port::user_repo::*;
use ::user::types::UserId;
use ::uuid::Uuid;

type UserRow = (Uuid, String);
type UserPasswordHashRow = (Uuid, Vec<u8>);
type UserAndHashRow = (Uuid, String, Vec<u8>);

impl UserRepo for Sqlite {
    async fn get_user_by_id(
        &self,
        id: &UserId,
    ) -> Result<Option<User>, GetUserByIdError> {
        let row: Option<(String,)> =
            ::sqlx::query_as("SELECT name FROM users WHERE id = ?")
                .bind(id.byte_slice())
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| GetUserByIdError {
                    source: Box::new(e),
                    id: *id,
                })?;

        Ok(row.map(|(username,)| User {
            id: *id,
            username: username.into(),
        }))
    }

    async fn get_user_by_name(
        &self,
        uname: &str,
    ) -> Result<Option<User>, GetUserByNameError> {
        let row: Option<UserRow> = ::sqlx::query_as(
            "SELECT id, name FROM users WHERE name = ? COLLATE NOCASE",
        )
        .bind(uname)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GetUserByNameError {
            source: Box::new(e),
            username: uname.into(),
        })?;

        let Some((id, username)) = row else {
            return Ok(None);
        };

        Ok(Some(User {
            id: UserId::try_from(id).map_err(|e| GetUserByNameError {
                source: Box::new(e),
                username: uname.into(),
            })?,
            username: username.into(),
        }))
    }

    async fn insert_user_with_hash(
        &self,
        user: &User,
        hash: &UserPasswordHash,
    ) -> Result<(), InsertUserWithHashError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| InsertUserWithHashError::Unknown(e.into()))?;

        sqlx::query("INSERT INTO users (id, name) VALUES (?, ?)")
            .bind(user.id.byte_slice())
            .bind(user.username.as_str())
            .execute(&mut *tx)
            .await
            .map_err(|err| {
                if let Some(db_err) = err.as_database_error() {
                    if db_err.code().as_deref() == Some("1555") {
                        return InsertUserWithHashError::IdTaken { id: user.id };
                    }

                    if db_err.code().as_deref() == Some("2067") {
                        return InsertUserWithHashError::NameTaken {
                            username: user.username.clone(),
                        };
                    }
                }

                InsertUserWithHashError::Unknown(err.into())
            })?;

        sqlx::query(
            "INSERT INTO user_password_hashes (user_id, password_hash) VALUES (?, ?)",
        )
        .bind(user.id.byte_slice())
        .bind(hash.hash.as_bytes())
        .execute(&mut *tx)
        .await
        .map_err(|e| InsertUserWithHashError::Unknown(e.into()))?;

        tx.commit()
            .await
            .map_err(|e| InsertUserWithHashError::Unknown(e.into()))
    }

    async fn get_user_and_hash_by_name(
        &self,
        uname: &str,
    ) -> Result<Option<(User, UserPasswordHash)>, GetUserAndHashByNameError> {
        let row: Option<UserAndHashRow> = sqlx::query_as(
            "SELECT u.id, u.name, h.password_hash FROM users u \
             JOIN user_password_hashes h ON u.id = h.user_id \
             WHERE u.name = ? COLLATE NOCASE",
        )
        .bind(uname)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GetUserAndHashByNameError {
            source: Box::new(e),
            uname: uname.into(),
        })?;

        let Some((id, username, hash)) = row else {
            return Ok(None);
        };

        let user_id =
            UserId::try_from(id).map_err(|e| GetUserAndHashByNameError {
                source: Box::new(e),
                uname: uname.into(),
            })?;

        let user = User {
            id: user_id,
            username: username.into(),
        };

        let hash = UserPasswordHash {
            user_id,
            hash: hash.into(),
        };

        Ok(Some((user, hash)))
    }

    async fn update_user_password_hash(
        &self,
        hash: &UserPasswordHash,
    ) -> Result<(), UpdateUserPasswordHashError> {
        let result = sqlx::query(
            "UPDATE user_password_hashes SET password_hash = ? WHERE user_id = ?",
        )
        .bind(hash.hash.as_bytes())
        .bind(hash.user_id.byte_slice())
        .execute(&self.pool)
        .await
        .map_err(|e| UpdateUserPasswordHashError::Unknown(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(UpdateUserPasswordHashError::UserNotFound {
                id: hash.user_id,
            });
        }

        Ok(())
    }

    async fn get_hash_by_user_id(
        &self,
        user_id: &UserId,
    ) -> Result<Option<UserPasswordHash>, GetHashByUserIdError> {
        let row: Option<UserPasswordHashRow> = sqlx::query_as(
            "SELECT user_id, password_hash FROM user_password_hashes WHERE user_id = ?",
        )
        .bind(user_id.byte_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GetHashByUserIdError {
            source: Box::new(e),
            user_id: *user_id,
        })?;

        let Some((user_uuid, hash)) = row else {
            return Ok(None);
        };

        let user_id =
            UserId::try_from(user_uuid).map_err(|e| GetHashByUserIdError {
                source: Box::new(e),
                user_id: *user_id,
            })?;

        Ok(Some(UserPasswordHash {
            user_id,
            hash: hash.into(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::fake::{Fake, Faker};
    use ::sqlx::SqlitePool;

    fn user_id() -> UserId {
        "01890f4d2e3b7e3a8c1d9b2a4f6e1d2c".parse().unwrap()
    }

    #[sqlx::test(fixtures("../fixtures/single-user.sql"))]
    async fn get_user_by_id_success_some(pool: SqlitePool) {
        let repo = Sqlite::new(pool);
        let id = user_id();
        let Some(user) = repo.get_user_by_id(&id).await.unwrap() else {
            panic!("User not found");
        };
        assert_eq!(user.id, id);
        assert_eq!(user.username, "bman");
    }

    #[sqlx::test]
    async fn get_user_by_id_success_none(pool: SqlitePool) {
        let repo = Sqlite::new(pool);
        let id = user_id();
        let user = repo.get_user_by_id(&id).await.unwrap();
        assert!(user.is_none());
    }

    #[sqlx::test(fixtures("../fixtures/single-user.sql"))]
    async fn get_user_by_name_success_some(pool: SqlitePool) {
        let repo = Sqlite::new(pool);
        let Some(user) = repo.get_user_by_name("bman").await.unwrap() else {
            panic!("User not found");
        };
        assert_eq!(user.id, user_id());
        assert_eq!(user.username, "bman");
    }

    #[sqlx::test(fixtures("../fixtures/single-user.sql"))]
    async fn get_user_by_name_with_different_case_success_some(pool: SqlitePool) {
        let repo = Sqlite::new(pool);
        let Some(user) = repo.get_user_by_name("BMAN").await.unwrap() else {
            panic!("User not found");
        };
        assert_eq!(user.id, user_id());
        assert_eq!(user.username, "bman");
    }

    #[sqlx::test]
    async fn insert_user_with_hash_success(pool: SqlitePool) {
        let repo = Sqlite::new(pool);
        let user = Faker.fake::<User>();
        let hash = UserPasswordHash {
            user_id: user.id,
            hash: Faker.fake(),
        };
        repo.insert_user_with_hash(&user, &hash).await.unwrap();
    }

    #[sqlx::test(fixtures("../fixtures/single-user.sql"))]
    async fn get_user_and_hash_by_name_success_some(pool: SqlitePool) {
        let repo = Sqlite::new(pool);
        let user_id = user_id();

        let Some((user, hash)) =
            repo.get_user_and_hash_by_name("bman").await.unwrap()
        else {
            panic!("User not found");
        };

        assert_eq!(user.id, user_id);
        assert_eq!(user.username, "bman");
        assert_eq!(hash.user_id, user_id);
        assert_eq!(hash.hash.as_bytes(), user_id.byte_slice());
    }

    #[sqlx::test(fixtures("../fixtures/single-user.sql"))]
    async fn update_user_password_hash_success(pool: SqlitePool) {
        let repo = Sqlite::new(pool);
        let hash = UserPasswordHash {
            user_id: user_id(),
            hash: Faker.fake(),
        };
        repo.update_user_password_hash(&hash).await.unwrap();
    }

    #[sqlx::test]
    async fn update_user_password_hash_not_found(pool: SqlitePool) {
        let repo = Sqlite::new(pool);
        let hash = Faker.fake::<UserPasswordHash>();
        let err = repo.update_user_password_hash(&hash).await.unwrap_err();
        assert!(matches!(
            err,
            UpdateUserPasswordHashError::UserNotFound { id } if id == hash.user_id
        ));
    }

    #[sqlx::test(fixtures("../fixtures/single-user.sql"))]
    async fn get_hash_by_user_id_success_some(pool: SqlitePool) {
        let repo = Sqlite::new(pool);
        let user_id = user_id();
        let Some(hash) = repo.get_hash_by_user_id(&user_id).await.unwrap() else {
            panic!("Hash not found");
        };
        assert_eq!(hash.user_id, user_id);
        assert_eq!(hash.hash.as_bytes(), user_id.byte_slice());
    }

    #[sqlx::test]
    async fn get_hash_by_user_id_success_none(pool: SqlitePool) {
        let repo = Sqlite::new(pool);
        let hash = repo.get_hash_by_user_id(&user_id()).await.unwrap();
        assert!(hash.is_none());
    }

    #[tokio::test]
    async fn user_repo_test_suite() {
        let repo = Sqlite::in_memory().await;
        repo.migrate().await.unwrap();
        test_adapter(&repo)
            .await
            .expect("UserRepo test suite should pass");
    }
}
