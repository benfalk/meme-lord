use crate::config::Config;
use actix_web::{http::header, HttpRequest};
use meme_lord_core::FileStore;

#[derive(Debug, Clone)]
pub struct State {
    pub config: Config,
    pub store: FileStore,
}

impl State {
    pub fn new(config: Config) -> Self {
        let store = FileStore::new(config.directory.clone());

        Self { store, config }
    }

    pub fn can_add_meme(&self, req: &HttpRequest) -> bool {
        if self.config.secret.is_empty() {
            return true;
        }

        match req.headers().get(header::AUTHORIZATION) {
            None => false,
            Some(auth) => auth.to_str().unwrap_or_default() == self.config.secret,
        }
    }
}
