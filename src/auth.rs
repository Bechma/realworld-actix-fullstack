use actix_web::{http::StatusCode, HttpResponse};

use crate::routing::ROUTES;

const AUTHOR_NAME: &'static str = "name";

pub fn redirect_to_profile(session: &actix_session::Session) -> Option<HttpResponse> {
    if let Ok(x) = get_username(session) {
        if let Some(name) = x {
            return Some(
                HttpResponse::build(StatusCode::FOUND)
                    .insert_header((
                        actix_web::http::header::LOCATION,
                        format!("{}/{}", ROUTES["profile"], name),
                    ))
                    .finish(),
            );
        }
    }
    None
}

pub fn is_authenticated(session: &actix_session::Session) -> bool {
    if let Ok(x) = get_username(session) {
        x.is_some()
    } else {
        false
    }
}

pub fn set_cookie_param(
    session: &actix_session::Session,
    name: String,
) -> Result<(), actix_session::SessionInsertError> {
    session.insert(AUTHOR_NAME, name)
}

pub fn get_username(
    session: &actix_session::Session,
) -> Result<Option<String>, actix_session::SessionGetError> {
    session.get(AUTHOR_NAME)
}
