use crate::meme::MemeDetails;
use crate::store::Store;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Manifest {
    server: String,
    memes: Vec<MemeDetails>,
}

impl Manifest {
    pub fn new<S: Store>(server: String, store: &S) -> Self {
        Self {
            server,
            memes: store.details(),
        }
    }
}
