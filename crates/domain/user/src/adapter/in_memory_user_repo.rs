use crate::entity::{User, UserPasswordHash};
use crate::port::user_repo::*;
use crate::types::{UserId, Username};
use ::std::collections::HashMap;
use ::std::sync::Arc;
use ::tokio::sync::RwLock;

#[derive(Clone, Default)]
pub struct InMemoryUserRepo {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Default)]
struct Inner {
    users: HashMap<UserId, User>,
    by_name: HashMap<Username, UserId>,
    hashes: HashMap<UserId, UserPasswordHash>,
}

impl UserRepo for InMemoryUserRepo {
    async fn get_user_by_id(
        &self,
        id: &UserId,
    ) -> Result<Option<User>, GetUserByIdError> {
        let inner = self.inner.read().await;
        Ok(inner.users.get(id).cloned())
    }

    async fn get_user_by_name(
        &self,
        name: &str,
    ) -> Result<Option<User>, GetUserByNameError> {
        let inner = self.inner.read().await;
        let Some(id) = inner.by_name.get(name) else {
            return Ok(None);
        };
        Ok(inner.users.get(id).cloned())
    }

    async fn get_user_and_hash_by_name(
        &self,
        name: &str,
    ) -> Result<Option<(User, UserPasswordHash)>, GetUserAndHashByNameError> {
        let inner = self.inner.read().await;
        let Some(id) = inner.by_name.get(name) else {
            return Ok(None);
        };
        let Some(user) = inner.users.get(id).cloned() else {
            return Ok(None);
        };
        let Some(hash) = inner.hashes.get(id).cloned() else {
            return Ok(None);
        };
        Ok(Some((user, hash)))
    }

    async fn insert_user_with_hash(
        &self,
        user: &User,
        hash: &UserPasswordHash,
    ) -> Result<(), InsertUserWithHashError> {
        let mut inner = self.inner.write().await;

        match inner.users.get(&user.id) {
            None => (),
            Some(existing_user) if existing_user == user => {
                return Err(InsertUserWithHashError::Duplicate {
                    id: existing_user.id,
                    username: existing_user.username.clone(),
                });
            }
            Some(existing_user) if existing_user.id == user.id => {
                return Err(InsertUserWithHashError::IdTaken { id: user.id });
            }
            Some(unexpected) => {
                return Err(InsertUserWithHashError::Unknown(::anyhow::anyhow!(
                    "unexpected user with id {} and name {}",
                    unexpected.id,
                    unexpected.username
                )));
            }
        }

        if inner.by_name.contains_key(&user.username) {
            return Err(InsertUserWithHashError::NameTaken {
                username: user.username.clone(),
            });
        }

        inner.users.insert(user.id, user.clone());
        inner.by_name.insert(user.username.clone(), user.id);
        inner.hashes.insert(user.id, hash.clone());
        Ok(())
    }

    async fn update_user_password_hash(
        &self,
        hash: &UserPasswordHash,
    ) -> Result<(), UpdateUserPasswordHashError> {
        let mut inner = self.inner.write().await;
        let Some(existing) = inner.hashes.get_mut(&hash.user_id) else {
            return Err(UpdateUserPasswordHashError::UserNotFound {
                id: hash.user_id,
            });
        };
        existing.hash = hash.hash.clone();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn passes_adapter_tests() {
        let repo = InMemoryUserRepo::default();
        test_adapter(&repo)
            .await
            .expect("InMemoryUserRepo to pass adapter tests");
    }
}
