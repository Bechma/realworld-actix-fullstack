use actix_web::{
    dev::{ServiceFactory, ServiceRequest, ServiceResponse},
    App,
};
use sqlx::{postgres::PgPoolOptions, Executor};

pub async fn get_test_pool() -> sqlx::PgPool {
    let p = PgPoolOptions::new()
        .connect("postgres://postgres:postgres@localhost/postgres")
        .await
        .expect("database connection can't be done");
    p.execute("CREATE EXTENSION IF NOT EXISTS pgcrypto")
        .await
        .expect("pgcrypto not available");
    p
}

#[allow(unused_must_use)]
pub async fn create_server() -> App<
    impl ServiceFactory<
        ServiceRequest,
        Config = (),
        Response = ServiceResponse,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    let state = std::sync::Arc::new(realworld_rust_fullstack::AppStateStruct::new({
        let mut tera =
            tera::Tera::new("templates/**/*").expect("Parsing error while loading template folder");
        tera.autoescape_on(vec!["j2"]);
        tera
    }));
    let configure = state.apply_routes();
    let pool = get_test_pool().await;
    sqlx::migrate!().undo(&pool, i64::MAX).await.unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();

    let session_key = actix_web::cookie::Key::from("1".repeat(512).as_bytes());
    App::new()
        .app_data(actix_web::web::Data::new(pool.clone()))
        .app_data(actix_web::web::Data::new(state.clone()))
        .wrap(
            actix_session::SessionMiddleware::builder(
                actix_session::storage::CookieSessionStore::default(),
                session_key,
            )
            .cookie_name("session".into())
            .cookie_secure(false)
            .build(),
        )
        .configure(configure)
}
