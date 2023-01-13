use super::ROUTES;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PathDelInfo {
    slug: String,
    id: i32,
}

pub async fn comments_delete(
    session: actix_session::Session,
    path_params: web::Path<PathDelInfo>,
    pool: web::Data<sqlx::PgPool>,
) -> impl Responder {
    remove_comment(session, &path_params, pool).await;

    HttpResponse::build(StatusCode::FOUND)
        .insert_header((
            actix_web::http::header::LOCATION,
            format!("{}/{}", ROUTES["article"], path_params.slug),
        ))
        .finish()
}

async fn remove_comment(
    session: actix_session::Session,
    path_params: &web::Path<PathDelInfo>,
    pool: web::Data<sqlx::PgPool>,
) -> Option<sqlx::Error> {
    let username = crate::auth::get_session_username(&session)?;
    let mut conn = pool.acquire().await.unwrap();
    sqlx::query!(
        "DELETE FROM Comments WHERE id=$1 and article=$2 and username=$3",
        path_params.id,
        path_params.slug,
        username,
    )
    .execute(&mut conn)
    .await
    .err()
}

#[derive(Deserialize)]
pub struct PathCreateInfo {
    slug: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentsForm {
    body: String,
}

pub async fn comments_create(
    session: actix_session::Session,
    path_params: web::Path<PathCreateInfo>,
    article_form: web::Form<CommentsForm>,
    pool: web::Data<sqlx::PgPool>,
) -> impl Responder {
    let comment = create_comment(session, &path_params.slug, &article_form.body, pool)
        .await
        .map(|x| format!("#comment-{x}"))
        .unwrap_or_default();

    HttpResponse::build(StatusCode::FOUND)
        .insert_header((
            actix_web::http::header::LOCATION,
            format!("{}/{}{}", ROUTES["article"], path_params.slug, comment),
        ))
        .finish()
}

async fn create_comment(
    session: actix_session::Session,
    slug: &str,
    body: &str,
    pool: web::Data<sqlx::PgPool>,
) -> Option<i32> {
    if body.is_empty() {
        return None;
    }
    let username = crate::auth::get_session_username(&session)?;
    let mut conn = pool.acquire().await.unwrap();
    Some(
        sqlx::query!(
            "INSERT INTO Comments(article, username, body) VALUES ($1, $2, $3) RETURNING id",
            slug,
            username,
            body,
        )
        .fetch_one(&mut conn)
        .await
        .unwrap()
        .id,
    )
}
