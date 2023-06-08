use crate::state::AppStateStruct;
use actix_web::middleware::Logger;

mod routing;
mod state;
mod utils;
use sqlx::Executor;

use actix_web::web::ServiceConfig;
use anyhow::Context;
use shuttle_actix_web::ShuttleActixWeb;

#[shuttle_runtime::main]
async fn actix_web(
    #[shuttle_shared_db::Postgres] pool: sqlx::PgPool,
    #[shuttle_secrets::Secrets] secret_store: shuttle_secrets::SecretStore,
    #[shuttle_static_folder::StaticFolder(folder = "templates")] static_folder: std::path::PathBuf,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    pool.execute("CREATE EXTENSION IF NOT EXISTS pgcrypto")
        .await
        .context("cannot create required extension")?;

    sqlx::migrate!()
        .run(&pool)
        .await
        .context("cannot generate migrations")?;

    let secret = secret_store
        .get("COOKIE_SECRET")
        .context("secret was not found")?;

    let state = std::sync::Arc::new(AppStateStruct::new({
        let mut tera = tera::Tera::new(
            &(static_folder
                .to_str()
                .context("cannot get static folder")?
                .to_string()
                + "/**/*"),
        )
        .context("Parsing error while loading template folder")?;
        tera.autoescape_on(vec!["j2"]);
        tera
    }));

    let session_key = actix_web::cookie::Key::from(secret.as_bytes());

    let config = move |cfg: &mut ServiceConfig| {
        let state = state.clone();
        let conf = state.apply_routes();
        cfg.service(
            actix_web::web::scope("")
                .app_data(actix_web::web::Data::new(pool.clone()))
                .app_data(actix_web::web::Data::new(state))
                .wrap(
                    actix_session::SessionMiddleware::builder(
                        actix_session::storage::CookieSessionStore::default(),
                        session_key.clone(),
                    )
                    .cookie_name("session".into())
                    .cookie_secure(true)
                    .build(),
                )
                .wrap(Logger::default())
                .configure(conf),
        );
    };

    Ok(config.into())
}
