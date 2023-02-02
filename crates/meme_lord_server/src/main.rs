mod args;
mod config;
mod meme_input;
mod state;
mod web_server;

use clap::Parser;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let args = args::Args::parse();
    web_server::run(args.config()).await
}
