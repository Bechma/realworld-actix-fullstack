const AUTHOR_NAME: &str = "name";

pub fn set_cookie_param(
    session: &actix_session::Session,
    name: String,
) -> Result<(), actix_session::SessionInsertError> {
    session.insert(AUTHOR_NAME, name)
}

pub fn get_session_username(session: &actix_session::Session) -> Option<String> {
    session.get(AUTHOR_NAME).unwrap_or_default()
}
