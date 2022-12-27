use super::db_models::{ArticlePreview, User};
use super::ROUTES;
use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PathInfo {
    username: String,
}

#[derive(Deserialize)]
pub struct QueryInfo {
    favourites: Option<bool>,
}

pub async fn user_profile(
    session: actix_session::Session,
    path_params: web::Path<PathInfo>,
    query_params: web::Query<QueryInfo>,
    pool: web::Data<sqlx::PgPool>,
) -> impl Responder {
    let mut conn = pool.acquire().await.unwrap();
    let logged_user = crate::auth::get_session_username(&session).unwrap_or_default();

    let Some(user) = sqlx::query!(
        "SELECT username, email, bio, image, EXISTS(SELECT 1 FROM Follows WHERE follower=$2 and influencer=$1) as following FROM Users where username=$1",
        path_params.username,
        logged_user.clone(),
    ).map(|x| User{ username: x.username, email: x.email, bio: x.bio, image: x.image, following: x.following.unwrap_or_default() })
    .fetch_optional(&mut conn)
    .await
    .unwrap() else {
        return HttpResponse::NotFound().finish();
    };

    let favourites = query_params.favourites.is_some();

    // I couldn't make this smaller... sadge
    let articles = if favourites {
        sqlx::query!(
            "
SELECT 
    a.slug,
    a.title,
    a.description,
    a.created_at,
    (SELECT COUNT(*) FROM FavArticles WHERE article=a.slug) as favorites_count,
    EXISTS(SELECT 1 FROM FavArticles WHERE article=a.slug and username=$2) as fav,
    EXISTS(SELECT 1 FROM Follows WHERE follower=$2 and influencer=a.author) as following,
    (SELECT string_agg(tag, ' ') FROM ArticleTags WHERE article = a.slug) as tag_list
FROM Articles as a
    JOIN FavArticles as fa ON fa.article = a.slug and fa.username = $1",
            path_params.username,
            logged_user,
        )
        .map(|x| ArticlePreview {
            slug: x.slug,
            title: x.title,
            fav: x.fav.unwrap_or_default(),
            description: x.description,
            created_at: x.created_at.format("%d/%m/%Y %H:%M").to_string(),
            favorites_count: x.favorites_count,
            tags: x.tag_list.unwrap_or_default(),
            author: User {
                username: user.username.to_string(),
                email: user.email.to_string(),
                bio: user.bio.clone(),
                image: user.image.clone(),
                following: x.following.unwrap_or_default(),
            },
        })
        .fetch_all(&mut conn)
        .await
        .unwrap()
    } else {
        sqlx::query!(
            "
SELECT 
    a.slug,
    a.title,
    a.description,
    a.created_at,
    (SELECT COUNT(*) FROM FavArticles WHERE article=a.slug) as favorites_count,
    EXISTS(SELECT 1 FROM FavArticles WHERE article=a.slug and username=$2) as fav,
    EXISTS(SELECT 1 FROM Follows WHERE follower=$2 and influencer=a.author) as following,
    (SELECT string_agg(tag, ' ') FROM ArticleTags WHERE article = a.slug) as tag_list
FROM Articles as a
WHERE a.author = $1",
            path_params.username,
            logged_user,
        )
        .map(|x| ArticlePreview {
            slug: x.slug,
            title: x.title,
            fav: x.fav.unwrap_or_default(),
            description: x.description,
            created_at: x.created_at.format("%d/%m/%Y %H:%M").to_string(),
            favorites_count: x.favorites_count,
            tags: x.tag_list.unwrap_or_default(),
            author: User {
                username: user.username.to_string(),
                email: user.email.to_string(),
                bio: user.bio.clone(),
                image: user.image.clone(),
                following: x.following.unwrap_or_default(),
            },
        })
        .fetch_all(&mut conn)
        .await
        .unwrap()
    };

    let mut context = tera::Context::new();
    context.insert(
        "current",
        format!("{}/{}", ROUTES["profile"], path_params.username).as_str(),
    );
    context.insert("user", &user);
    context.insert("articles", &articles);
    context.insert("favourites", &query_params.favourites.is_some());

    crate::template::render_template("profile.j2", session, &mut context)
}

pub async fn follower_up(
    session: actix_session::Session,
    path_params: web::Path<PathInfo>,
    request: actix_web::HttpRequest,
    pool: web::Data<sqlx::PgPool>,
) -> impl Responder {
    if let Some(username) = crate::auth::get_session_username(&session) {
        let mut conn = pool.acquire().await.unwrap();
        sqlx::query!(
            "INSERT INTO Follows(follower, influencer) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            username,
            path_params.username,
        )
        .execute(&mut conn)
        .await
        .unwrap();
    }

    HttpResponse::build(StatusCode::FOUND)
        .insert_header((
            actix_web::http::header::LOCATION,
            request
                .headers()
                .get(actix_web::http::header::REFERER)
                .map(|x| x.to_str().unwrap().to_string())
                .unwrap_or_else(|| {
                    ROUTES["profile"].to_string() + "/" + path_params.username.as_str()
                }),
        ))
        .finish()
}

pub async fn follower_down(
    session: actix_session::Session,
    path_params: web::Path<PathInfo>,
    request: actix_web::HttpRequest,
    pool: web::Data<sqlx::PgPool>,
) -> impl Responder {
    if let Some(username) = crate::auth::get_session_username(&session) {
        let mut conn = pool.acquire().await.unwrap();
        sqlx::query!(
            "DELETE FROM Follows WHERE follower=$1 and influencer=$2",
            username,
            path_params.username,
        )
        .execute(&mut conn)
        .await
        .unwrap();
    }

    HttpResponse::build(StatusCode::FOUND)
        .insert_header((
            actix_web::http::header::LOCATION,
            request
                .headers()
                .get(actix_web::http::header::REFERER)
                .map(|x| x.to_str().unwrap().to_string())
                .unwrap_or_else(|| {
                    ROUTES["profile"].to_string() + "/" + path_params.username.as_str()
                }),
        ))
        .finish()
}
