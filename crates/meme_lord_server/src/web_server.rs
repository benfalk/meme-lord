mod handler;
use crate::state::State;
use actix_web::{web, App, HttpServer};

pub async fn run() -> std::io::Result<()> {
    let state = State::default();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/.manifest", web::get().to(handler::manifest))
            .route("/add", web::post().to(handler::add_meme))
            .route("/{id}", web::get().to(handler::get_meme))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
