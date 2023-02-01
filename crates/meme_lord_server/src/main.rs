mod config;
mod meme_input;
mod state;
mod web_server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    web_server::run().await
}
