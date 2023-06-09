pub async fn error_handler(
    session: actix_session::Session,
    state: actix_web::web::Data<crate::state::AppState>,
) -> super::ConduitResponse {
    state.render_template("error.j2", &session, &mut tera::Context::new())
}
