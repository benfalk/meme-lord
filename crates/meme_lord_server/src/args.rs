use clap::Parser;

use crate::config::Config;

/// WebServer for Dank Memes
#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Directory to store meme data
    #[arg(short, long, default_value = "/tmp")]
    dir: String,

    /// if set, secret is required to upload new memes
    #[arg(short, long, default_value = "")]
    secret: String,

    /// Server root for manifest
    #[arg(short, long, default_value = "http://localhost:8080/")]
    root: String,
}

impl Args {
    pub fn config(&self) -> Config {
        Config {
            directory: self.dir.clone(),
            server: self.root.clone(),
            secret: self.secret.clone(),
        }
    }
}
