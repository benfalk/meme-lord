use crate::entity::Meme;
use crate::port::meme_repo::*;
use crate::types::{MemeId, MemePath};
use ::std::collections::HashMap;
use ::std::sync::Arc;
use ::tokio::sync::RwLock;

#[derive(Debug, Default)]
pub struct InMemoryMemeRepo {
    by_id: Arc<RwLock<HashMap<MemeId, Arc<Meme>>>>,
    by_path: Arc<RwLock<HashMap<MemePath, Arc<Meme>>>>,
}

impl MemeRepo for InMemoryMemeRepo {
    async fn delete(&self, id: &MemeId) -> Result<(), DeleteByIdMemeError> {
        let mut id_write = self.by_id.write().await;

        let Some(found) = id_write.remove(id) else {
            return Err(DeleteByIdMemeError::MemeNotFound { id: *id });
        };

        let mut path_write = self.by_path.write().await;
        path_write.remove(&found.path);
        Ok(())
    }

    async fn insert(&self, meme: &Meme) -> Result<(), InsertMemeError> {
        let stored = Arc::new(meme.clone());

        let mut id_write = self.by_id.write().await;

        if id_write.contains_key(&meme.id) {
            return Err(InsertMemeError::MemeAlreadyExists { id: meme.id });
        }

        let mut path_write = self.by_path.write().await;

        if path_write.contains_key(&meme.path) {
            return Err(InsertMemeError::PathTaken {
                path: meme.path.clone(),
            });
        }

        id_write.insert(meme.id, Arc::clone(&stored));
        path_write.insert(meme.path.clone(), stored);

        Ok(())
    }

    async fn update_by_id(&self, meme: &Meme) -> Result<(), UpdateByIdMemeError> {
        let mut id_write = self.by_id.write().await;

        let Some(current) = id_write.get_mut(&meme.id) else {
            return Err(UpdateByIdMemeError::MemeNotFound { id: meme.id });
        };

        let mut path_write = self.by_path.write().await;

        if current.path != meme.path {
            if path_write.contains_key(&meme.path) {
                return Err(UpdateByIdMemeError::PathTaken {
                    path: meme.path.clone(),
                });
            }
            path_write.remove(&current.path);
        }

        *Arc::make_mut(current) = meme.clone();
        path_write.insert(current.path.clone(), Arc::clone(current));

        Ok(())
    }

    async fn fetch_by_id(&self, id: &MemeId) -> Result<Meme, FetchByIdError> {
        let id_read = self.by_id.read().await;
        let Some(found) = id_read.get(id) else {
            return Err(FetchByIdError::MemeNotFound { id: *id });
        };
        Ok((**found).clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_meme_repo() -> Result<(), MemeRepoError> {
        let repo = InMemoryMemeRepo::default();
        test_meme_repo(&repo)
            .await
            .expect("in memory meme repo to pass");
        Ok(())
    }
}
