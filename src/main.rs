use std::os::unix::prelude::OsStrExt;

use crate::routing::apply_routes;
use actix_web::{middleware::Logger, App, HttpServer};

mod auth;
mod routing;
mod template;
use env_logger::Env;
use sqlx::{postgres::PgPoolOptions, Executor};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().expect("dotenv is not loaded");
    env_logger::init_from_env(Env::default().default_filter_or("debug"));
    let pool = PgPoolOptions::new()
        .connect(&std::env::var("DATABASE_URL").expect("DATABASE_URL environment is not set"))
        .await
        .expect("database connection can't be done");
    pool.execute("CREATE EXTENSION IF NOT EXISTS pgcrypto")
        .await
        .expect("pgcrypto not available");
    sqlx::migrate!().run(&pool).await.expect("migrations done");

    let session_key = actix_web::cookie::Key::from(
        std::env::var_os("COOKIE_SECRET")
            .expect("cookie secret not set")
            .as_bytes(),
    );

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
