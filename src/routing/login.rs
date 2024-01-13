use actix_web::web;
use actix_web::web::Data;
use serde::Deserialize;

pub async fn login_get(
    session: actix_session::Session,
    state: Data<crate::state::AppState>,
) -> super::ConduitResponse {
    if let Some(x) = super::redirect_to_self_profile(&session) {
        return Ok(x);
    }
    login_template(session, false, state).await
}

#[derive(Deserialize)]
pub struct FormData {
    username: String,
    password: String,
}

pub async fn login_post(
    session: actix_session::Session,
    form_data: web::Form<FormData>,
    pool: Data<sqlx::PgPool>,
    state: Data<crate::state::AppState>,
) -> super::ConduitResponse {
    if let Some(x) = super::redirect_to_self_profile(&session) {
        return Ok(x);
    }
    if !sqlx::query!(
        "SELECT username FROM Users where username=$1 and password=crypt($2, password)",
        form_data.username,
        form_data.password
    )
    .fetch_all(pool.as_ref())
    .await
    .unwrap_or_default()
    .is_empty()
    {
        crate::utils::set_cookie_param(&session, form_data.username.to_string());
        return Ok(crate::utils::redirect(format!(
            "{}/{}",
            super::RoutesEnum::Profile,
            form_data.username
        )));
    }
    login_template(session, true, state).await
}

async fn login_template(
    session: actix_session::Session,
    error: bool,
    state: Data<crate::state::AppState>,
) -> super::ConduitResponse {
    let mut context = tera::Context::new();
    if error {
        context.insert("error", "invalid user");
    }
    state.render_template("login.j2", &session, &mut context)
}
