use actix_web::{
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    App,
};
use realworld_rust_fullstack::apply_routes;
use sqlx::{postgres::PgPoolOptions, Executor};

pub async fn get_test_pool() -> sqlx::PgPool {
    let p = PgPoolOptions::new()
        .connect(&"postgres://postgres:postgres@localhost/postgres")
        .await
        .expect("database connection can't be done");
    p.execute("CREATE EXTENSION IF NOT EXISTS pgcrypto")
        .await
        .expect("pgcrypto not available");
    p
}

pub async fn create_server() -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let pool = get_test_pool().await;
    sqlx::migrate!().undo(&pool, i64::MAX).await.unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();

    let session_key = actix_web::cookie::Key::from("1".repeat(512).as_bytes());
    App::new()
        .app_data(actix_web::web::Data::new(pool.clone()))
        .wrap(
            actix_session::SessionMiddleware::builder(
                actix_session::storage::CookieSessionStore::default(),
                session_key.clone(),
            )
            .cookie_name("session".into())
            .cookie_secure(false)
            .build(),
        )
        .configure(apply_routes)
}
