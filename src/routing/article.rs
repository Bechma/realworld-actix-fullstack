use actix_web::{http::StatusCode, web, HttpResponse, Responder};
use serde::Deserialize;

use super::{
    db_models::{ArticleFull, Comments, User},
    ROUTES,
};

#[derive(Deserialize)]
pub struct PathInfo {
    slug: String,
}

pub async fn article(
    session: actix_session::Session,
    path_params: web::Path<PathInfo>,
    pool: web::Data<sqlx::PgPool>,
) -> impl Responder {
    let mut conn = pool.acquire().await.unwrap();
    let Ok(article) = get_article(&mut conn, &path_params.slug).await else {
        return HttpResponse::NotFound().finish();
    };
    let mut context = tera::Context::new();
    context.insert("article", &article);
    if let Some(username) = crate::auth::get_session_username(&session) {
        let user = sqlx::query_as!(
            User,
            "SELECT username, email, bio, image FROM Users WHERE username=$1",
            username
        )
        .fetch_optional(&mut conn)
        .await
        .unwrap();
        context.insert("user", &user);
    }
    if let Ok(comments) = get_comments(&mut conn, &path_params.slug).await {
        context.insert("comments", &comments);
        println!("{:?}", comments);
    }
    crate::template::render_template("article.j2", session, &mut context)
}

async fn get_article(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    slug: &str,
) -> Result<ArticleFull, sqlx::Error> {
    sqlx::query!(
        "
SELECT
    a.*,
    (SELECT string_agg(tag, ' ') FROM ArticleTags WHERE article = a.slug) as tag_list,
    (SELECT COUNT(*) FROM FavArticles WHERE article = a.slug) as fav_count,
    u.*
FROM Articles a
    JOIN Users u ON a.author = u.username
WHERE slug = $1
",
        slug
    )
    .map(|x| ArticleFull {
        slug: x.slug,
        title: x.title,
        description: x.description,
        body: x.body,
        tag_list: x
            .tag_list
            .unwrap()
            .split_ascii_whitespace()
            .map(str::to_string)
            .collect::<Vec<_>>(),
        favorites_count: x.fav_count.unwrap(),
        created_at: x.created_at.format("%d/%m/%Y %H:%M").to_string(),
        author: User {
            username: x.username,
            email: x.email,
            bio: x.bio,
            image: x.image,
        },
    })
    .fetch_one(conn)
    .await
}

async fn get_comments(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    slug: &str,
) -> Result<Vec<Comments>, sqlx::Error> {
    sqlx::query!(
        "
SELECT c.*, u.image as user_image FROM Comments c
    JOIN Users u ON c.username=u.username
WHERE article=$1",
        slug
    )
    .map(|x| Comments {
        id: x.id,
        user_link: ROUTES["profile"].to_string() + "/" + x.username.as_str(),
        article: x.article,
        username: x.username,
        body: x.body,
        created_at: x.created_at.format("%d/%m/%Y %H:%M").to_string(),
        user_image: x.user_image.unwrap_or_default(),
    })
    .fetch_all(conn)
    .await
}

pub async fn article_delete(
    session: actix_session::Session,
    path_params: web::Path<PathInfo>,
    pool: web::Data<sqlx::PgPool>,
) -> impl Responder {
    if let Some(username) = crate::auth::get_session_username(&session) {
        let mut conn = pool.acquire().await.unwrap();
        sqlx::query!(
            "DELETE FROM Articles WHERE slug=$1 and author=$2",
            path_params.slug,
            username,
        )
        .execute(&mut conn)
        .await
        .unwrap();
    }

    HttpResponse::build(StatusCode::FOUND)
        .insert_header((
            actix_web::http::header::LOCATION,
            ROUTES["index"].to_string(),
        ))
        .finish()
}
