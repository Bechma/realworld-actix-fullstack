mod auth;
mod routing;
mod template;

use actix_web::web::ServiceConfig;
use anyhow::anyhow;
use shuttle_service::ShuttleActixWeb;
use sqlx::Executor;

pub use crate::routing::apply_routes;

#[shuttle_service::main]
async fn actix_web(
    #[shuttle_shared_db::Postgres] pool: sqlx::PgPool,
    #[shuttle_secrets::Secrets] secret_store: shuttle_secrets::SecretStore,
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
    let session_key = actix_web::cookie::Key::from(
        (0..secret.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&secret[i..i + 2], 16).unwrap())
            .collect::<Vec<u8>>()
            .as_slice(),
    );

    Ok(move |cfg: &mut ServiceConfig| {
        cfg.service(
            actix_web::web::scope("/")
                .wrap(
                    actix_session::SessionMiddleware::builder(
                        actix_session::storage::CookieSessionStore::default(),
                        session_key.clone(),
                    )
                    .cookie_name("session".into())
                    .cookie_secure(true)
                    .build(),
                )
                .configure(apply_routes),
        );
    })
}
