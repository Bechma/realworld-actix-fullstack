use super::ROUTES;
use actix_web::http::{header, StatusCode};
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

use super::db_models::User;

pub async fn settings_get(
    session: actix_session::Session,
    pool: web::Data<sqlx::PgPool>,
) -> impl Responder {
    if let Some(username) = crate::auth::get_session_username(&session) {
        let mut conn = pool.acquire().await.unwrap();
        let user = sqlx::query!(
            "SELECT username, email, bio, image FROM Users WHERE username=$1",
            username
        )
        .map(|x| User {
            username: x.username,
            email: x.email,
            bio: x.bio,
            image: x.image,
            following: false,
        })
        .fetch_one(&mut conn)
        .await
        .unwrap();

        let mut context = tera::Context::new();
        context.insert("user", &user);
        return crate::template::render_template("settings.j2", session, &mut context);
    }
    HttpResponse::Found()
        .append_header((header::LOCATION, ROUTES["login"].as_str()))
        .finish()
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormData {
    image: String,
    bio: String,
    email: String,
    password: String,
    confirm_password: String,
}

pub async fn settings_post(
    session: actix_session::Session,
    form_data: web::Form<FormData>,
    pool: web::Data<sqlx::PgPool>,
) -> impl Responder {
    if let Some(username) = crate::auth::get_session_username(&session) {
        let change_password = if !form_data.password.is_empty() {
            if form_data.password != form_data.confirm_password {
                let mut context = tera::Context::new();
                let user = User {
                    username: username,
                    email: form_data.email.to_string(),
                    bio: Some(form_data.bio.to_string()),
                    image: Some(form_data.image.to_string()),
                    following: false,
                };
                context.insert("user", &user);
                return crate::template::render_template("settings.j2", session, &mut context);
            }
            true
        } else {
            false
        };

        let mut conn = pool.acquire().await.unwrap();
        sqlx::query!(
            "
UPDATE Users SET
    image=$2,
    bio=$3,
    email=$4,
    password=CASE WHEN $5 IS TRUE THEN crypt($6, gen_salt('bf')) ELSE password END
WHERE username=$1",
            username,
            form_data.image,
            form_data.bio,
            form_data.email,
            change_password,
            form_data.password
        )
        .execute(&mut conn)
        .await
        .unwrap();
    }

    HttpResponse::build(StatusCode::FOUND)
        .append_header((actix_web::http::header::LOCATION, ROUTES["index"].as_str()))
        .finish()
}
