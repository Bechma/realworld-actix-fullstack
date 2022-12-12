use super::ROUTES;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

pub async fn register_get(session: actix_session::Session) -> impl Responder {
    if let Some(x) = crate::auth::redirect_to_profile(&session) {
        return x;
    }
    let mut context = tera::Context::new();
    crate::template::render_template("register.j2", session, &mut context)
}

#[derive(Deserialize)]
pub struct FormData {
    username: String,
    email: String,
    password: String,
}

pub async fn register_post(
    session: actix_session::Session,
    form_data: web::Form<FormData>,
    pool: Data<sqlx::PgPool>,
) -> impl Responder {
    if let Some(x) = crate::auth::redirect_to_profile(&session) {
        return x;
    }
    // sqlx::Pool<sqlx::Postgres>
    // PgPool
    let mut transaction = match pool.begin().await {
        Ok(x) => x,
        Err(_) => {
            return HttpResponse::build(StatusCode::FOUND)
                .insert_header((
                    actix_web::http::header::LOCATION,
                    ROUTES["register"].as_str(),
                ))
                .finish()
        }
    };
    if sqlx::query!(
        "INSERT INTO Users(username, email, password) VALUES ($1, $2, crypt($3, gen_salt('bf')))",
        form_data.username,
        form_data.email,
        form_data.password
    )
    .execute(&mut transaction)
    .await
    .is_ok()
        && transaction.commit().await.is_ok()
    {
        crate::auth::set_cookie_param(&session, form_data.username.to_string()).unwrap();
        return HttpResponse::build(StatusCode::FOUND)
            .insert_header((
                actix_web::http::header::LOCATION,
                format!("{}/{}", ROUTES["profile"], form_data.username),
            ))
            .finish();
    }
    let mut context = tera::Context::new();
    context.insert("error", "User already registered");
    crate::template::render_template("register.j2", session, &mut context)
}
