use self::article::{article, article_add_favorite, article_del_favorite, article_delete};
use self::comments::{comments_create, comments_delete};
use self::editor::{editor_get, editor_post};
use self::error::error_handler;
use self::index::index;
use self::login::{login_get, login_post};
use self::logout::logout;
use self::profile::{follower_down, follower_up, user_profile};
use self::register::{register_get, register_post};
use self::settings::{settings_get, settings_post};
use actix_web::web;

mod article;
mod comments;
mod db_models;
mod editor;
mod error;
mod index;
mod login;
mod logout;
mod profile;
mod register;
mod settings;

pub type ConduitResponse = Result<actix_web::HttpResponse, crate::errors::ConduitError>;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct Routes {
    index: String,
    logout: String,
    login: String,
    register: String,
    settings: String,
    editor: String,
    article: String,
    profile: String,
    error: String,
}

pub(crate) enum RoutesEnum {
    Index,
    // Logout,
    Login,
    Register,
    // Settings,
    // Editor,
    Article,
    Profile,
    // Error,
}

impl Routes {
    pub fn new() -> Self {
        Self {
            index: "/".to_string(),
            logout: "/logout".to_string(),
            login: "/login".to_string(),
            register: "/register".to_string(),
            settings: "/settings".to_string(),
            editor: "/editor".to_string(),
            article: "/article".to_string(),
            profile: "/profile".to_string(),
            error: "/error".to_string(),
        }
    }

    pub(crate) fn enum_to_string(&self, value: &RoutesEnum) -> String {
        match value {
            RoutesEnum::Index => self.index.to_string(),
            // RoutesEnum::Logout => self.logout.to_string(),
            RoutesEnum::Login => self.login.to_string(),
            RoutesEnum::Register => self.register.to_string(),
            // RoutesEnum::Settings => self.settings.to_string(),
            // RoutesEnum::Editor => self.editor.to_string(),
            RoutesEnum::Article => self.article.to_string(),
            RoutesEnum::Profile => self.profile.to_string(),
        }
    }

    pub fn redirect_to_profile(
        &self,
        session: &actix_session::Session,
    ) -> Option<actix_web::HttpResponse> {
        crate::utils::get_session_username(session).map(|name| {
            actix_web::HttpResponse::build(actix_web::http::StatusCode::FOUND)
                .insert_header((
                    actix_web::http::header::LOCATION,
                    format!("{}/{}", self.profile, name),
                ))
                .finish()
        })
    }

    pub fn apply_routes(&self) -> impl Fn(&mut web::ServiceConfig) {
        let s = self.clone();
        move |cfg: &mut web::ServiceConfig| {
            let editor_slug = s.editor.to_string() + "/{slug}";
            let article_slug = s.article.to_string() + "/{slug}";
            let profile_user = s.profile.to_string() + "/{username}";
            let article_slug_delete = article_slug.to_string() + "/delete";
            let article_add_comment = article_slug.to_string() + "/comments";
            let article_del_comment = article_add_comment.to_string() + "/{id}";
            let article_fav = article_slug.to_string() + "/fav";
            let article_unfav = article_slug.to_string() + "/unfav";
            let user_follow = profile_user.to_string() + "/follow";
            let user_unfollow = profile_user.to_string() + "/unfollow";
            cfg.route(&s.index, web::get().to(index))
                .route(&article_slug, web::get().to(article))
                .route(&article_slug_delete, web::post().to(article_delete))
                .route(&article_add_comment, web::post().to(comments_create))
                .route(&article_del_comment, web::post().to(comments_delete))
                .route(&article_fav, web::post().to(article_add_favorite))
                .route(&article_unfav, web::post().to(article_del_favorite))
                .route(&s.logout, web::post().to(logout))
                .route(&s.login, web::get().to(login_get))
                .route(&s.login, web::post().to(login_post))
                .route(&s.register, web::get().to(register_get))
                .route(&s.register, web::post().to(register_post))
                .route(&s.settings, web::get().to(settings_get))
                .route(&s.settings, web::post().to(settings_post))
                .route(&s.editor, web::get().to(editor_get))
                .route(&s.editor, web::post().to(editor_post))
                .route(&editor_slug, web::get().to(editor_get))
                .route(&editor_slug, web::post().to(editor_post))
                .route(&profile_user, web::get().to(user_profile))
                .route(&user_follow, web::post().to(follower_up))
                .route(&user_unfollow, web::post().to(follower_down))
                .route(&s.error, web::get().to(error_handler));
        }
    }
}
