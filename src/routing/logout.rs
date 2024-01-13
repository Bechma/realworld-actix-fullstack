pub async fn logout(session: actix_session::Session) -> actix_web::HttpResponse {
    session.clear();
    crate::utils::redirect(super::RoutesEnum::Index.to_string())
}
