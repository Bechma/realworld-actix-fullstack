use actix_web::{http::StatusCode, web, HttpResponse};
use serde::Deserialize;

use super::db_models::{ArticleFull, Comments, User};

#[derive(Deserialize)]
pub struct PathInfo {
    slug: String,
}

pub async fn article(
    session: actix_session::Session,
    path_params: web::Path<PathInfo>,
    pool: web::Data<sqlx::PgPool>,
    state: web::Data<crate::state::AppState>,
) -> super::ConduitResponse {
    let mut conn = pool.acquire().await?;
    let username = crate::utils::get_session_username(&session);
    let Ok(article) = get_article(&mut conn, &path_params.slug, username.clone().unwrap_or_default()).await else {
        return Ok(HttpResponse::NotFound().finish());
    };
    let mut context = tera::Context::new();
    context.insert("article", &article);
    if let Some(username) = username {
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
        .fetch_optional(&mut conn)
        .await?;
        context.insert("user", &user);
    }
    let profile_route = state.route_from_enum(&super::RoutesEnum::Profile);
    if let Ok(comments) = get_comments(&mut conn, &path_params.slug, profile_route).await {
        context.insert("comments", &comments);
    }
    state.render_template("article.j2", &session, &mut context)
}

async fn get_article(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    slug: &str,
    username: String,
) -> Result<ArticleFull, sqlx::Error> {
    sqlx::query!(
        "
SELECT
    a.*,
    (SELECT string_agg(tag, ' ') FROM ArticleTags WHERE article = a.slug) as tag_list,
    (SELECT COUNT(*) FROM FavArticles WHERE article = a.slug) as fav_count,
    u.*,
    EXISTS(SELECT 1 FROM FavArticles WHERE article=a.slug and username=$2) as fav,
    EXISTS(SELECT 1 FROM Follows WHERE follower=$2 and influencer=a.author) as following
FROM Articles a
    JOIN Users u ON a.author = u.username
WHERE slug = $1
",
        slug,
        username,
    )
    .map(|x| ArticleFull {
        slug: x.slug,
        title: x.title,
        description: x.description,
        body: x.body,
        tag_list: x
            .tag_list
            .unwrap_or_default()
            .split_ascii_whitespace()
            .map(str::to_string)
            .collect::<Vec<_>>(),
        favorites_count: x.fav_count.unwrap_or_default(),
        created_at: x.created_at.format("%d/%m/%Y %H:%M").to_string(),
        fav: x.fav.unwrap_or_default(),
        author: User {
            username: x.username,
            email: x.email,
            bio: x.bio,
            image: x.image,
            following: x.following.unwrap_or_default(),
        },
    })
    .fetch_one(conn)
    .await
}

async fn get_comments(
    conn: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    slug: &str,
    profile_route: String,
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
        user_link: profile_route.to_string() + "/" + x.username.as_str(),
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
    state: web::Data<crate::state::AppState>,
) -> super::ConduitResponse {
    if let Some(username) = crate::utils::get_session_username(&session) {
        let mut conn = pool.acquire().await?;
        sqlx::query!(
            "DELETE FROM Articles WHERE slug=$1 and author=$2",
            path_params.slug,
            username,
        )
        .execute(&mut conn)
        .await?;
    }

    Ok(HttpResponse::build(StatusCode::FOUND)
        .insert_header((
            actix_web::http::header::LOCATION,
            state.route_from_enum(&super::RoutesEnum::Index),
        ))
        .finish())
}

pub async fn article_add_favorite(
    session: actix_session::Session,
    path_params: web::Path<PathInfo>,
    request: actix_web::HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    state: web::Data<crate::state::AppState>,
) -> super::ConduitResponse {
    if let Some(username) = crate::utils::get_session_username(&session) {
        let mut conn = pool.acquire().await?;
        sqlx::query!(
            "INSERT INTO FavArticles(article, username) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            path_params.slug,
            username,
        )
        .execute(&mut conn)
        .await?;
    }

    Ok(HttpResponse::build(StatusCode::FOUND)
        .insert_header((
            actix_web::http::header::LOCATION,
            request
                .headers()
                .get(actix_web::http::header::REFERER)
                .map_or_else(
                    || {
                        state.route_from_enum(&super::RoutesEnum::Article)
                            + "/"
                            + path_params.slug.as_str()
                    },
                    |x| x.to_str().unwrap_or_default().to_string(),
                ),
        ))
        .finish())
}

pub async fn article_del_favorite(
    session: actix_session::Session,
    path_params: web::Path<PathInfo>,
    request: actix_web::HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    state: web::Data<crate::state::AppState>,
) -> super::ConduitResponse {
    if let Some(username) = crate::utils::get_session_username(&session) {
        let mut conn = pool.acquire().await?;
        sqlx::query!(
            "DELETE FROM FavArticles WHERE article=$1 and username=$2",
            path_params.slug,
            username,
        )
        .execute(&mut conn)
        .await?;
    }

    Ok(HttpResponse::build(StatusCode::FOUND)
        .insert_header((
            actix_web::http::header::LOCATION,
            request
                .headers()
                .get(actix_web::http::header::REFERER)
                .map_or_else(
                    || {
                        state.route_from_enum(&super::RoutesEnum::Article)
                            + "/"
                            + path_params.slug.as_str()
                    },
                    |x| x.to_str().unwrap_or_default().to_string(),
                ),
        ))
        .finish())
}
