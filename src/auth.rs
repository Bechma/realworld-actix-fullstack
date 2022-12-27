use actix_web::{http::StatusCode, HttpResponse};

use crate::routing::ROUTES;

const AUTHOR_NAME: &str = "name";

pub fn redirect_to_profile(session: &actix_session::Session) -> Option<HttpResponse> {
    if let Some(name) = get_session_username(session) {
        return Some(
            HttpResponse::build(StatusCode::FOUND)
                .insert_header((
                    actix_web::http::header::LOCATION,
                    format!("{}/{}", ROUTES["profile"], name),
                ))
                .finish(),
        );
    }
    None
}

pub fn set_cookie_param(
    session: &actix_session::Session,
    name: String,
) -> Result<(), actix_session::SessionInsertError> {
    session.insert(AUTHOR_NAME, name)
}

pub fn get_session_username(session: &actix_session::Session) -> Option<String> {
    if let Ok(x) = session.get(AUTHOR_NAME) {
        return x;
    }
    None
}
