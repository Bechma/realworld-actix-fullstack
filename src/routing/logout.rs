use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::HttpResponse;

pub async fn logout(
    session: actix_session::Session,
    state: Data<crate::state::AppState>,
) -> HttpResponse {
    session.clear();
    HttpResponse::build(StatusCode::FOUND)
        .insert_header((
            actix_web::http::header::LOCATION,
            state.route_from_enum(&super::RoutesEnum::Index),
        ))
        .finish()
}
