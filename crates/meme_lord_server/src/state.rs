use meme_lord_core::FileStore;
use crate::config::Config;

#[derive(Debug, Clone)]
pub struct State {
    pub config: Config,
    pub store: FileStore,
}

impl State {
    pub fn new(config: Config) -> Self {
        let store = FileStore::new(config.directory.clone());

        Self {
            store,
            config,
        }
    }
}
