use crate::entity::Meme;
use crate::port::meme_repo::*;
use crate::types::{MemeId, MemePath};
use ::std::collections::HashMap;
use ::std::sync::Arc;
use ::tokio::sync::RwLock;

pub struct InMemoryMemeRepo {
    by_id: Arc<RwLock<HashMap<MemeId, Arc<Meme>>>>,
    by_path: Arc<RwLock<HashMap<MemePath, Arc<Meme>>>>,
}

impl MemeRepo for InMemoryMemeRepo {
    async fn delete(&self, id: &MemeId) -> Result<(), DeleteByIdMemeError> {
        todo!()
    }

    async fn insert(&self, meme: &Meme) -> Result<(), InsertMemeError> {
        todo!()
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
        todo!()
    }
}
