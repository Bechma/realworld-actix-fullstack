use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

use super::db_models::{ArticleFull, User};

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
    let article = if let Ok(x) = sqlx::query!(
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
        path_params.slug
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
    .fetch_one(&mut conn)
    .await
    {
        x
    } else {
        return HttpResponse::NotFound().finish();
    };
    let mut context = tera::Context::new();
    context.insert("article", &article);
    crate::template::render_template("article.j2", session, &mut context)
}
