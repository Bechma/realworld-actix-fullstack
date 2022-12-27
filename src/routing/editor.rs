use std::collections::HashSet;

use super::ROUTES;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

use super::db_models::ArticleEdit;

#[derive(Deserialize)]
pub struct PathInfo {
    slug: Option<String>,
}

pub async fn editor_get(
    session: actix_session::Session,
    path_params: web::Path<PathInfo>,
    pool: web::Data<sqlx::PgPool>,
) -> impl Responder {
    if !crate::auth::get_session_username(&session).is_some() {
        return HttpResponse::build(StatusCode::FOUND)
            .insert_header((actix_web::http::header::LOCATION, ROUTES["login"].as_str()))
            .finish();
    }
    let article = if let Some(slug) = &path_params.slug {
        let mut conn = pool.acquire().await.unwrap();
        if let Some(x) = sqlx::query!(
            "
SELECT
    a.*,
    (SELECT string_agg(tag, ' ') FROM ArticleTags WHERE article = a.slug) as tag_list
FROM Articles a WHERE slug = $1",
            slug
        )
        .map(|x| ArticleEdit {
            slug: x.slug,
            title: x.title,
            description: x.description,
            body: x.body,
            tag_list: x.tag_list.unwrap_or_default(),
            author: x.author,
        })
        .fetch_optional(&mut conn)
        .await
        .unwrap()
        {
            x
        } else {
            return HttpResponse::NotFound().finish();
        }
    } else {
        ArticleEdit::default()
    };
    let mut context = tera::Context::new();
    context.insert("article", &article);
    crate::template::render_template("editor.j2", session, &mut context)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleForm {
    title: String,
    description: String,
    body: String,
    tag_list: String,
}

const BIND_LIMIT: usize = 65535;

pub async fn editor_post(
    session: actix_session::Session,
    path_params: web::Path<PathInfo>,
    article_form: web::Form<ArticleForm>,
    pool: web::Data<sqlx::PgPool>,
) -> impl Responder {
    let slug = if let Some(author) = crate::auth::get_session_username(&session) {
        update_article(author, path_params, article_form, pool)
            .await
            .unwrap()
    } else {
        // Not authenticated
        return HttpResponse::build(StatusCode::FOUND)
            .insert_header((actix_web::http::header::LOCATION, ROUTES["index"].as_str()))
            .finish();
    };

    HttpResponse::build(StatusCode::FOUND)
        .append_header((
            actix_web::http::header::LOCATION,
            format!("{}/{}", ROUTES["article"], slug),
        ))
        .finish()
}

async fn update_article(
    author: String,
    path_params: web::Path<PathInfo>,
    article_form: web::Form<ArticleForm>,
    pool: web::Data<sqlx::PgPool>,
) -> Result<String, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let slug = if let Some(slug) = &path_params.slug {
        sqlx::query!(
            "UPDATE Articles SET title=$1, description=$2, body=$3 WHERE slug=$4",
            article_form.title,
            article_form.description,
            article_form.body,
            slug
        )
        .execute(&mut transaction)
        .await?;
        slug.to_string()
    } else {
        let slug = article_form.title.to_lowercase().replace(" ", "-");
        sqlx::query!(
            "INSERT INTO Articles(slug, title, description, body, author) VALUES ($1, $2, $3, $4, $5)",
            slug,
            article_form.title,
            article_form.description,
            article_form.body,
            author
        )
        .execute(&mut transaction)
        .await?;
        slug
    };
    sqlx::query!("DELETE FROM ArticleTags WHERE article=$1", slug)
        .execute(&mut transaction)
        .await?;
    let article_tags = article_form
        .tag_list
        .trim()
        .split_ascii_whitespace()
        .collect::<HashSet<&str>>();
    if !article_tags.is_empty() {
        let mut qb = sqlx::QueryBuilder::new("INSERT INTO ArticleTags(article, tag) ");
        qb.push_values(
            article_tags.into_iter().take(BIND_LIMIT / 2),
            |mut b, tag| {
                b.push_bind(slug.clone()).push_bind(tag);
            },
        );
        qb.build().execute(&mut transaction).await?;
    }

    transaction.commit().await?;
    Ok(slug)
}
