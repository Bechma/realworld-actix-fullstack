const AUTHOR_NAME: &str = "name";

pub fn set_cookie_param(session: &actix_session::Session, name: String) {
    let _ = session.insert(AUTHOR_NAME, name);
}

pub fn get_session_username(session: &actix_session::Session) -> Option<String> {
    session.get(AUTHOR_NAME).unwrap_or_default()
}

pub fn redirect(path: String) -> actix_web::HttpResponse {
    actix_web::HttpResponse::build(actix_web::http::StatusCode::FOUND)
        .insert_header((actix_web::http::header::LOCATION, path))
        .finish()
}
