use super::db_models::{ArticlePreview, User};
use super::ROUTES;
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
    let user: Option<User> = sqlx::query_as!(
        User,
        "SELECT username, email, bio, image FROM Users where username=$1",
        path_params.username
    )
    .fetch_optional(&mut conn)
    .await
    .unwrap();

    let user = match user {
        Some(x) => x,
        None => return HttpResponse::NotFound().finish(),
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
    (SELECT COUNT(*) FROM FavArticles WHERE article = a.slug) as favorites_count
FROM Articles as a
    JOIN FavArticles as fa ON fa.article = a.slug and fa.username = $1",
            path_params.username
        )
        .map(|x| ArticlePreview {
            slug: x.slug,
            title: x.title,
            description: x.description,
            created_at: x.created_at.format("%d/%m/%Y %H:%M").to_string(),
            favorites_count: x.favorites_count,
            author: user.clone(),
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
    (SELECT COUNT(*) FROM FavArticles WHERE slug = a.slug) as favorites_count
FROM Articles as a
WHERE a.author = $1",
            path_params.username
        )
        .map(|x| ArticlePreview {
            slug: x.slug,
            title: x.title,
            description: x.description,
            created_at: x.created_at.format("%d/%m/%Y %H:%M").to_string(),
            favorites_count: x.favorites_count,
            author: user.clone(),
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

/*
pub fn follow(
    session: actix_session::Session,
    path_params: web::Path<PathInfo>,
    query_params: web::Query<QueryInfo>,
    pool: web::Data<sqlx::PgPool>,
) -> impl Responder {
    path_params.username
}
*/
