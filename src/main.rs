#![deny(clippy::unwrap_used, clippy::pedantic)]
#![allow(clippy::unused_async, clippy::module_name_repetitions)]
use crate::state::AppStateStruct;
use actix_web::HttpResponse;
use actix_web::middleware::Logger;
use std::env;

mod errors;
mod routing;
mod state;
mod utils;

#[actix_web::get("/assets/main.css")]
async fn main_css() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/css")
        .body(include_str!("main.css"))
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let cookie_secret = env::var("COOKIE_SECRET").expect("COOKIE_SECRET must be set");
    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Cannot run migrations");

    let session_key = actix_web::cookie::Key::from(cookie_secret.as_bytes());

    let bind_address = format!("{host}:{port}");
    println!("Starting server at http://{bind_address}");

    actix_web::HttpServer::new(move || {
        let state = std::sync::Arc::new(AppStateStruct::new({
            let mut tera = tera::Tera::new("templates/**/*")
                .expect("Parsing error while loading template folder");
            tera.autoescape_on(vec!["j2"]);
            tera
        }));
        let conf = state.apply_routes();
        actix_web::App::new()
            .app_data(actix_web::web::Data::new(pool.clone()))
            .app_data(actix_web::web::Data::new(state.clone()))
            .wrap(
                actix_session::SessionMiddleware::builder(
                    actix_session::storage::CookieSessionStore::default(),
                    session_key.clone(),
                )
                .cookie_name("session".into())
                .cookie_secure(false)
                .build(),
            )
            .wrap(Logger::default())
            .service(main_css)
            .configure(conf)
    })
    .bind(&bind_address)?
    .run()
    .await
}
