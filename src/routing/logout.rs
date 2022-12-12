use super::ROUTES;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, Responder};

pub async fn logout(session: actix_session::Session) -> impl Responder {
    session.clear();
    HttpResponse::build(StatusCode::FOUND)
        .insert_header((actix_web::http::header::LOCATION, ROUTES["index"].as_str()))
        .finish()
}
