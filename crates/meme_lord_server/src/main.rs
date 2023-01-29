mod meme_input;
mod state;

use actix_web::{web, App, HttpServer};
use state::State;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let state = State::default();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(state.clone()))
            .route("/.manifest", web::get().to(handler::manifest))
            .route("/add", web::post().to(handler::add_meme))
            .route("/{id}", web::get().to(handler::get_meme))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

mod handler {
    use crate::meme_input::MemeInput;
    use crate::state::State;
    use actix_multipart::Multipart;
    use actix_web::{web, Error, HttpResponse, Responder};
    use meme_lord_core::{Manifest, Store};

    pub async fn manifest(data: web::Data<State>) -> impl Responder {
        let server = "http://localhost:8080/".to_owned();
        let manifest = Manifest::new(server, &data.store);
        serde_json::to_string(&manifest).unwrap()
    }

    pub async fn add_meme(
        data: Multipart,
        state: web::Data<State>,
    ) -> Result<impl Responder, Error> {
        let input = MemeInput::try_from(data).await?;

        if input.is_valid() {
            state.store.put(input.to_meme());
            Ok(HttpResponse::Ok())
        } else {
            Ok(HttpResponse::BadRequest())
        }
    }

    pub async fn get_meme(path: web::Path<String>, state: web::Data<State>) -> impl Responder {
        let id = path.into_inner();

        match state.store.get(&id) {
            Some(meme) => HttpResponse::Ok().body(meme.into_data()),
            None => HttpResponse::NotFound().into(),
        }
    }
}
