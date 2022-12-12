use crate::routing::apply_routes;
use actix_web::{middleware::Logger, App, HttpServer};

mod auth;
mod routing;
mod template;
use env_logger::Env;
use sqlx::postgres::PgPoolOptions;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().expect("dotenv is not loaded");
    env_logger::init_from_env(Env::default().default_filter_or("debug"));
    let pool = PgPoolOptions::new()
        .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL environment is not set"))
        .await
        .expect("database connection can't be done");
    sqlx::query!("CREATE EXTENSION IF NOT EXISTS pgcrypto")
        .execute(&pool)
        .await
        .expect("pgcrypto not available");
    let session_key =
        actix_web::cookie::Key::try_from("1".repeat(2048).as_bytes()).expect("secret");

    HttpServer::new(move || {
        App::new()
            .app_data(actix_web::web::Data::new(pool.clone()))
            .wrap(
                actix_session::SessionMiddleware::builder(
                    actix_session::storage::CookieSessionStore::default(),
                    session_key.clone(),
                )
                .cookie_name("session".into())
                .cookie_secure(std::env::var_os("PRODUCTION").is_some())
                .build(),
            )
            .wrap(Logger::default())
            .configure(apply_routes)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
