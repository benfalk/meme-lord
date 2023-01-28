use crate::meme::{Meme, MemeDetails};
use crate::store::Store;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

type MemeMap = HashMap<String, Meme>;
type ArcLock<T> = Arc<RwLock<T>>;

#[derive(Debug, Clone, Default)]
pub struct MemoryStore {
    data: ArcLock<MemeMap>,
}

impl Store for MemoryStore {
    fn get(&self, id: &str) -> Option<Meme> {
        self.data.read().unwrap().get(id).cloned()
    }

    fn put(&self, meme: Meme) {
        let id = meme.id().clone();
        self.data.write().unwrap().insert(id, meme);
    }

    fn details(&self) -> Vec<MemeDetails> {
        self.data
            .read()
            .unwrap()
            .values()
            .map(|m| m.details())
            .cloned()
            .collect()
    }
}
