use meme_lord_core::FileStore;

#[derive(Debug, Clone)]
pub struct State {
    pub store: FileStore,
}

impl Default for State {
    fn default() -> Self {
        Self {
            store: FileStore::new("/home/bfalk/meme-store"),
        }
    }
}
