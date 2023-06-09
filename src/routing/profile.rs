use super::db_models::{ArticlePreview, User};
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{web, HttpResponse};
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
    state: Data<crate::state::AppState>,
) -> super::ConduitResponse {
    let mut conn = pool.acquire().await?;
    let logged_user = crate::utils::get_session_username(&session).unwrap_or_default();

    let Some(user) = sqlx::query!(
        "SELECT username, email, bio, image, EXISTS(SELECT 1 FROM Follows WHERE follower=$2 and influencer=$1) as following FROM Users where username=$1",
        path_params.username,
        logged_user.clone(),
    ).map(|x| User{ username: x.username, email: x.email, bio: x.bio, image: x.image, following: x.following.unwrap_or_default() })
    .fetch_optional(&mut conn)
    .await? else {
        return Ok(HttpResponse::NotFound().finish());
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
    u.username,
    u.image,
    (SELECT COUNT(*) FROM FavArticles WHERE article=a.slug) as favorites_count,
    EXISTS(SELECT 1 FROM FavArticles WHERE article=a.slug and username=$2) as fav,
    EXISTS(SELECT 1 FROM Follows WHERE follower=$2 and influencer=a.author) as following,
    (SELECT string_agg(tag, ' ') FROM ArticleTags WHERE article = a.slug) as tag_list
FROM Articles as a
    JOIN Users as u ON u.username = a.author
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
                username: x.username,
                email: String::default(),
                bio: None,
                image: x.image,
                following: x.following.unwrap_or_default(),
            },
        })
        .fetch_all(&mut conn)
        .await?
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
                email: String::default(),
                bio: None,
                image: user.image.clone(),
                following: x.following.unwrap_or_default(),
            },
        })
        .fetch_all(&mut conn)
        .await?
    };

    let mut context = tera::Context::new();
    context.insert(
        "current",
        &(state.route_from_enum(&super::RoutesEnum::Profile) + "/" + &path_params.username),
    );
    context.insert("user", &user);
    context.insert("articles", &articles);
    context.insert("favourites", &query_params.favourites.is_some());

    state.render_template("profile.j2", &session, &mut context)
}

pub async fn follower_up(
    session: actix_session::Session,
    path_params: web::Path<PathInfo>,
    request: actix_web::HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    state: Data<crate::state::AppState>,
) -> super::ConduitResponse {
    if let Some(username) = crate::utils::get_session_username(&session) {
        let mut conn = pool.acquire().await?;
        sqlx::query!(
            "INSERT INTO Follows(follower, influencer) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            username,
            path_params.username,
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
                        state.route_from_enum(&super::RoutesEnum::Profile)
                            + "/"
                            + path_params.username.as_str()
                    },
                    |x| x.to_str().unwrap_or_default().to_string(),
                ),
        ))
        .finish())
}

pub async fn follower_down(
    session: actix_session::Session,
    path_params: web::Path<PathInfo>,
    request: actix_web::HttpRequest,
    pool: web::Data<sqlx::PgPool>,
    state: Data<crate::state::AppState>,
) -> super::ConduitResponse {
    if let Some(username) = crate::utils::get_session_username(&session) {
        let mut conn = pool.acquire().await?;
        sqlx::query!(
            "DELETE FROM Follows WHERE follower=$1 and influencer=$2",
            username,
            path_params.username,
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
                        state.route_from_enum(&super::RoutesEnum::Profile)
                            + "/"
                            + path_params.username.as_str()
                    },
                    |x| x.to_str().unwrap_or_default().to_string(),
                ),
        ))
        .finish())
}
