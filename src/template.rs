use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse,
};

use crate::routing::ROUTES;
lazy_static::lazy_static! {
    pub static ref TEMPLATES: once_cell::sync::OnceCell<tera::Tera> = once_cell::sync::OnceCell::new();
}

pub fn render_template(
    template: &str,
    session: actix_session::Session,
    context: &mut tera::Context,
) -> HttpResponse {
    let current = template.replace(".j2", "");
    if !context.contains_key("current") {
        context.insert("current", &ROUTES[current.as_str()]);
    }
    context.insert("routes", &ROUTES.clone());
    if let Some(username) = crate::auth::get_session_username(&session) {
        context.insert("username", &username);
    }
    match TEMPLATES.get().unwrap().render(template, &context) {
        Ok(body) => HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::html())
            .body(body),
        Err(e) => HttpResponse::build(StatusCode::BAD_REQUEST)
            .content_type(ContentType::plaintext())
            .body(e.to_string()),
    }
}
