use std::collections::HashMap;

use self::article::{article, article_delete};
use self::comments::{comments_create, comments_delete};
use self::editor::{editor_get, editor_post};
use self::index::index;
use self::login::{login_get, login_post};
use self::logout::logout;
use self::profile::user_profile;
use self::register::{register_get, register_post};
use self::settings::{settings_get, settings_post};
use actix_web::web;

mod article;
mod comments;
mod db_models;
mod editor;
mod index;
mod login;
mod logout;
mod profile;
mod register;
mod settings;

lazy_static::lazy_static!(
    pub static ref ROUTES: HashMap<String, String> = {
        let mut hm = HashMap::new();
        hm.insert("index", "/");
        hm.insert("logout", "/logout");
        hm.insert("login", "/login");
        hm.insert("register", "/register");
        hm.insert("settings", "/settings");
        hm.insert("editor", "/editor");
        hm.insert("article", "/article");
        hm.insert("profile", "/profile");
        hm.iter().map(|(x, y)| (x.to_string(), y.to_string())).collect()
    };
);

pub fn apply_routes(cfg: &mut web::ServiceConfig) {
    cfg.route(&ROUTES["index"], web::get().to(index))
        .route(
            &(ROUTES["article"].to_string() + "/{slug}"),
            web::get().to(article),
        )
        .route(
            &(ROUTES["article"].to_string() + "/{slug}/delete"),
            web::post().to(article_delete),
        )
        .route(
            &(ROUTES["article"].to_string() + "/{slug}/comments"),
            web::post().to(comments_create),
        )
        .route(
            &(ROUTES["article"].to_string() + "/{slug}/comments/{id}"),
            web::post().to(comments_delete),
        )
        .route(&ROUTES["logout"], web::get().to(logout))
        .route(&ROUTES["logout"], web::post().to(logout))
        .route(&ROUTES["login"], web::get().to(login_get))
        .route(&ROUTES["login"], web::post().to(login_post))
        .route(&ROUTES["register"], web::get().to(register_get))
        .route(&ROUTES["register"], web::post().to(register_post))
        .route(&ROUTES["settings"], web::get().to(settings_get))
        .route(&ROUTES["settings"], web::post().to(settings_post))
        .route(&ROUTES["editor"], web::get().to(editor_get))
        .route(
            &(ROUTES["editor"].to_string() + "/{slug}"),
            web::get().to(editor_get),
        )
        .route(&ROUTES["editor"], web::post().to(editor_post))
        .route(
            &(ROUTES["editor"].to_string() + "/{slug}"),
            web::post().to(editor_post),
        )
        .route(
            &(ROUTES["profile"].to_string() + "/{username}"),
            web::get().to(user_profile),
        );
}
