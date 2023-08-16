use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

pub async fn register_get(
    session: actix_session::Session,
    state: Data<crate::state::AppState>,
) -> super::ConduitResponse {
    if let Some(x) = state.redirect_to_profile(&session) {
        return Ok(x);
    }
    let mut context = tera::Context::new();
    state.render_template("register.j2", &session, &mut context)
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FormData {
    username: Option<String>,
    email: Option<String>,
    password: Option<String>,
}

struct SignUp {
    pub username: String,
    pub email: String,
    pub password: String,
}

pub async fn register_post(
    session: actix_session::Session,
    form_data: web::Form<FormData>,
    pool: Data<sqlx::PgPool>,
    state: Data<crate::state::AppState>,
) -> super::ConduitResponse {
    if let Some(x) = state.redirect_to_profile(&session) {
        return Ok(x);
    }
    let form_data = match validate_form(form_data.clone(), &state) {
        Ok(x) => x,
        Err(e) => {
            let mut context = tera::Context::new();
            context.insert("error", &e);
            context.insert("reg", &form_data);
            return state.render_template("register.j2", &session, &mut context);
        }
    };
    let Ok(mut transaction) = pool.begin().await else {
        return Ok(HttpResponse::build(StatusCode::FOUND)
            .insert_header((
                actix_web::http::header::LOCATION,
                state.route_from_enum(&super::RoutesEnum::Register),
            ))
            .finish())
    };
    if sqlx::query!(
        "INSERT INTO Users(username, email, password) VALUES ($1, $2, crypt($3, gen_salt('bf')))",
        form_data.username,
        form_data.email,
        form_data.password
    )
    .execute(transaction.as_mut())
    .await
    .is_ok()
        && transaction.commit().await.is_ok()
    {
        crate::utils::set_cookie_param(&session, form_data.username.to_string());
        return Ok(HttpResponse::build(StatusCode::FOUND)
            .insert_header((
                actix_web::http::header::LOCATION,
                format!(
                    "{}/{}",
                    state.route_from_enum(&super::RoutesEnum::Profile),
                    form_data.username
                ),
            ))
            .finish());
    }
    let mut context = tera::Context::new();
    context.insert("error", "User already registered");
    state.render_template("register.j2", &session, &mut context)
}

fn validate_form(
    form_data: FormData,
    state: &Data<crate::state::AppState>,
) -> Result<SignUp, String> {
    let username = form_data.username.unwrap_or_default();
    if username.len() < 4 {
        return Err("Username is too short, at least 4".into());
    }

    let email = form_data.email.unwrap_or_default();
    if !state.is_email(&email) {
        return Err("You need to provide an email address".into());
    }

    let password = form_data.password.unwrap_or_default();
    if password.is_empty() {
        return Err("You need to provide a password".into());
    }

    Ok(SignUp {
        username,
        email,
        password,
    })
}
