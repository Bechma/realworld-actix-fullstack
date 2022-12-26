mod auth;
mod routing;
mod template;

use actix_web::{middleware::Logger, web::ServiceConfig};
use anyhow::anyhow;
use shuttle_service::ShuttleActixWeb;
use sqlx::Executor;

pub use crate::routing::apply_routes;
pub use crate::template::TEMPLATES;

#[shuttle_service::main]
async fn actix_web(
    #[shuttle_shared_db::Postgres] pool: sqlx::PgPool,
    #[shuttle_secrets::Secrets] secret_store: shuttle_secrets::SecretStore,
    #[shuttle_static_folder::StaticFolder(folder = "templates")] static_folder: std::path::PathBuf,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Sync + Send + Clone + 'static> {
    pool.execute("CREATE EXTENSION IF NOT EXISTS pgcrypto")
        .await
        .map_err(|x| anyhow!(x.to_string()))?;

    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(|x| anyhow!(x.to_string()))?;

    let Some(secret) = secret_store.get("COOKIE_SECRET") else {
        return Err(anyhow!("secret was not found").into());
    };

    TEMPLATES
        .set({
            let mut tera =
                tera::Tera::new((static_folder.to_str().unwrap().to_string() + "/**/*").as_str())
                    .expect("Parsing error while loading template folder");
            tera.autoescape_on(vec!["j2"]);
            tera
        })
        .expect("tera instance couldn't initialize");

    let session_key = actix_web::cookie::Key::from(
        (0..secret.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&secret[i..i + 2], 16).unwrap())
            .collect::<Vec<u8>>()
            .as_slice(),
    );

    Ok(move |cfg: &mut ServiceConfig| {
        cfg.service(
            actix_web::web::scope("")
                .app_data(actix_web::web::Data::new(pool.clone()))
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
                .configure(apply_routes),
        );
    })
}
