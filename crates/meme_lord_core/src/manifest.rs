use crate::meme::MemeDetails;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Manifest {
    server: String,
    memes: Vec<MemeDetails>,
}
