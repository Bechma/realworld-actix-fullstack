use super::db_models::{ArticlePreview, User};
use actix_web::{web, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct QueryInfo {
    page: Option<u32>,
    amount: Option<u32>,
    tag: Option<String>,
    myfeed: Option<bool>,
}

pub async fn index(
    session: actix_session::Session,
    query_params: web::Query<QueryInfo>,
    pool: web::Data<sqlx::PgPool>,
) -> impl Responder {
    let mut conn = pool.acquire().await.unwrap();

    let page = query_params.page.unwrap_or(1).saturating_sub(1) as i64;
    let username = crate::auth::get_session_username(&session);
    let amount = query_params.amount.unwrap_or(10) as i64;
    let personal_feed = if query_params.myfeed.is_some() {
        username.clone().unwrap_or("%%".to_string())
    } else {
        "%%".to_string()
    };

    let articles = sqlx::query!(
        "
SELECT 
    a.slug,
    a.title,
    a.description,
    a.created_at,
    (SELECT COUNT(*) FROM FavArticles WHERE article=a.slug) as favorites_count,
    u.username, u.image,
    EXISTS(SELECT 1 FROM FavArticles WHERE article=a.slug and username=$5) as fav,
    EXISTS(SELECT 1 FROM Follows WHERE follower=$5 and influencer=u.username) as following
FROM Articles as a
    JOIN Users as u ON u.username LIKE $4 and a.author = u.username
WHERE
    a.slug in (SELECT distinct article FROM ArticleTags WHERE tag LIKE $3)
ORDER BY a.created_at desc
LIMIT $1 OFFSET $2",
        amount,
        page * amount,
        query_params.tag.clone().unwrap_or("%%".to_string()),
        personal_feed,
        username.unwrap_or_default(),
    )
    .map(|x| ArticlePreview {
        slug: x.slug,
        title: x.title,
        fav: x.fav.unwrap_or_default(),
        description: x.description,
        created_at: x.created_at.format("%d/%m/%Y %H:%M").to_string(),
        favorites_count: x.favorites_count,
        author: User {
            username: x.username,
            email: String::default(),
            bio: None,
            image: x.image,
            following: x.following.unwrap_or_default(),
        },
    })
    .fetch_all(&mut conn)
    .await
    .unwrap();

    let tags: Vec<String> = sqlx::query!("SELECT DISTINCT tag FROM ArticleTags")
        .map(|x| x.tag)
        .fetch_all(&mut conn)
        .await
        .unwrap();

    let mut context = tera::Context::new();
    context.insert("tags", &tags);
    context.insert("articles", &articles);
    context.insert("myfeed", &query_params.myfeed.is_some());
    crate::template::render_template("index.j2", session, &mut context)
}
