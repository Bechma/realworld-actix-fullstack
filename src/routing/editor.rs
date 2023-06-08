use std::collections::HashSet;

use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use super::db_models::ArticleEdit;

#[derive(Deserialize)]
pub struct PathInfo {
    slug: Option<String>,
}

pub async fn editor_get(
    session: actix_session::Session,
    path_params: web::Path<PathInfo>,
    pool: web::Data<sqlx::PgPool>,
    state: web::Data<crate::state::AppState>,
) -> impl Responder {
    if crate::utils::get_session_username(&session).is_none() {
        return HttpResponse::build(StatusCode::FOUND)
            .insert_header((
                actix_web::http::header::LOCATION,
                state.route_from_enum(super::RoutesEnum::Login),
            ))
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
    context.insert("slug", &article.slug);
    context.insert("article", &article);
    state
        .render_template("editor.j2", session, &mut context)
        .unwrap()
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ArticleForm {
    title: Option<String>,
    description: Option<String>,
    body: Option<String>,
    tag_list: Option<String>,
}

#[derive(Serialize)]
struct ArticleUpdate {
    title: String,
    description: String,
    body: String,
    tag_list: HashSet<String>,
}

const BIND_LIMIT: usize = 65535;

pub async fn editor_post(
    session: actix_session::Session,
    path_params: web::Path<PathInfo>,
    article_form: web::Form<ArticleForm>,
    pool: web::Data<sqlx::PgPool>,
    state: web::Data<crate::state::AppState>,
) -> impl Responder {
    let slug = if let Some(author) = crate::utils::get_session_username(&session) {
        let article = match validate_article(article_form.clone()) {
            Ok(x) => x,
            Err(e) => {
                let mut context = tera::Context::new();
                context.insert("slug", &path_params.slug);
                context.insert("article", &article_form);
                context.insert("error", &e);
                return state
                    .render_template("editor.j2", session, &mut context)
                    .unwrap();
            }
        };
        match update_article(author, &path_params, &article, pool).await {
            Ok(x) => x,
            Err(e) => {
                let mut context = tera::Context::new();
                context.insert("slug", &path_params.slug);
                context.insert("article", &article);
                if e.to_string().contains("duplicate key") {
                    context.insert(
                        "error",
                        "There's another article with that same title already",
                    );
                } else {
                    context.insert("error", "Problem during creation of the article");
                }
                return state
                    .render_template("editor.j2", session, &mut context)
                    .unwrap();
            }
        }
    } else {
        // Not authenticated
        return HttpResponse::build(StatusCode::FOUND)
            .insert_header((
                actix_web::http::header::LOCATION,
                state.route_from_enum(super::RoutesEnum::Index),
            ))
            .finish();
    };

    HttpResponse::build(StatusCode::FOUND)
        .append_header((
            actix_web::http::header::LOCATION,
            format!(
                "{}/{}",
                state.route_from_enum(super::RoutesEnum::Article),
                slug
            ),
        ))
        .finish()
}

fn validate_article(article_form: ArticleForm) -> Result<ArticleUpdate, String> {
    let title = article_form.title.unwrap_or_default();
    if title.len() < 4 {
        return Err("You need to provide a title with at least 4 characters".into());
    }

    let description = article_form.description.unwrap_or_default();
    if description.len() < 4 {
        return Err("You need to provide a description with at least 4 characters".into());
    }

    let body = article_form.body.unwrap_or_default();
    if body.len() < 10 {
        return Err("You need to provide a body with at least 10 characters".into());
    }

    let tag_list = article_form
        .tag_list
        .unwrap_or_default()
        .trim()
        .split_ascii_whitespace()
        .filter(|x| !x.is_empty())
        .map(str::to_string)
        .collect::<HashSet<String>>();
    Ok(ArticleUpdate {
        title,
        description,
        body,
        tag_list,
    })
}

async fn update_article(
    author: String,
    path_params: &web::Path<PathInfo>,
    article: &ArticleUpdate,
    pool: web::Data<sqlx::PgPool>,
) -> Result<String, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let slug = if let Some(slug) = &path_params.slug {
        sqlx::query!(
            "UPDATE Articles SET title=$1, description=$2, body=$3 WHERE slug=$4 and author=$5",
            article.title,
            article.description,
            article.body,
            slug,
            author,
        )
        .execute(&mut transaction)
        .await?;
        slug.to_string()
    } else {
        let slug = article.title.to_lowercase().replace(' ', "-");
        sqlx::query!(
            "INSERT INTO Articles(slug, title, description, body, author) VALUES ($1, $2, $3, $4, $5)",
            slug,
            article.title,
            article.description,
            article.body,
            author
        )
        .execute(&mut transaction)
        .await?;
        slug
    };
    sqlx::query!("DELETE FROM ArticleTags WHERE article=$1", slug)
        .execute(&mut transaction)
        .await?;
    if !article.tag_list.is_empty() {
        let mut qb = sqlx::QueryBuilder::new("INSERT INTO ArticleTags(article, tag) ");
        qb.push_values(
            article.tag_list.clone().into_iter().take(BIND_LIMIT / 2),
            |mut b, tag| {
                b.push_bind(slug.clone()).push_bind(tag);
            },
        );
        qb.build().execute(&mut transaction).await?;
    }

    transaction.commit().await?;
    Ok(slug)
}
