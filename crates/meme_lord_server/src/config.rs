use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub(crate) directory: String,
    pub(crate) server: String,
    pub(crate) secret: String,
}
