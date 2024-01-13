use actix_web::{web, HttpResponse};
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
) -> super::ConduitResponse {
    remove_comment(session, &path_params, pool).await?;

    Ok(crate::utils::redirect(format!(
        "{}/{}",
        super::RoutesEnum::Article,
        path_params.slug
    )))
}

async fn remove_comment(
    session: actix_session::Session,
    path_params: &web::Path<PathDelInfo>,
    pool: web::Data<sqlx::PgPool>,
) -> Result<u64, sqlx::Error> {
    let Some(username) = crate::utils::get_session_username(&session) else {
        return Ok(0);
    };
    sqlx::query!(
        "DELETE FROM Comments WHERE id=$1 and article=$2 and username=$3",
        path_params.id,
        path_params.slug,
        username,
    )
    .execute(pool.as_ref())
    .await
    .map(|x| x.rows_affected())
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
) -> HttpResponse {
    let comment = create_comment(session, &path_params.slug, &article_form.body, pool)
        .await
        .map(|x| format!("#comment-{x}"))
        .unwrap_or_default();

    crate::utils::redirect(format!(
        "{}/{}{}",
        super::RoutesEnum::Article,
        path_params.slug,
        comment
    ))
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
    let username = crate::utils::get_session_username(&session)?;
    Some(
        sqlx::query!(
            "INSERT INTO Comments(article, username, body) VALUES ($1, $2, $3) RETURNING id",
            slug,
            username,
            body,
        )
        .fetch_one(pool.as_ref())
        .await
        .ok()?
        .id,
    )
}
