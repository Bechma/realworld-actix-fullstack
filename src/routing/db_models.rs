use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct User {
    pub username: String,
    pub email: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ArticlePreview {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub created_at: String,
    pub favorites_count: Option<i64>,
    pub author: User,
}

#[derive(Serialize, Deserialize)]
pub struct ArticleFull {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub created_at: String,
    pub favorites_count: i64,
    pub tag_list: Vec<String>,
    pub author: User,
}

#[derive(Serialize, Deserialize, Default)]
pub struct ArticleEdit {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub tag_list: String,
    pub author: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Comments {
    pub id: i32,
    pub article: String,
    pub username: String,
    pub body: String,
    pub created_at: String,
    pub user_image: String,
    pub user_link: String,
}
