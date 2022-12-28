use super::ROUTES;
use actix_web::http::StatusCode;
use actix_web::web::Data;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

pub async fn register_get(session: actix_session::Session) -> impl Responder {
    if let Some(x) = crate::auth::redirect_to_profile(&session) {
        return x;
    }
    let mut context = tera::Context::new();
    crate::template::render_template("register.j2", session, &mut context)
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
) -> impl Responder {
    if let Some(x) = crate::auth::redirect_to_profile(&session) {
        return x;
    }
    let form_data = match validate_form(form_data.clone()) {
        Ok(x) => x,
        Err(e) => {
            let mut context = tera::Context::new();
            context.insert("error", &e);
            context.insert("reg", &form_data);
            return crate::template::render_template("register.j2", session, &mut context);
        }
    };
    let mut transaction = match pool.begin().await {
        Ok(x) => x,
        Err(_) => {
            return HttpResponse::build(StatusCode::FOUND)
                .insert_header((
                    actix_web::http::header::LOCATION,
                    ROUTES["register"].as_str(),
                ))
                .finish()
        }
    };
    if sqlx::query!(
        "INSERT INTO Users(username, email, password) VALUES ($1, $2, crypt($3, gen_salt('bf')))",
        form_data.username,
        form_data.email,
        form_data.password
    )
    .execute(&mut transaction)
    .await
    .is_ok()
        && transaction.commit().await.is_ok()
    {
        crate::auth::set_cookie_param(&session, form_data.username.to_string()).unwrap();
        return HttpResponse::build(StatusCode::FOUND)
            .insert_header((
                actix_web::http::header::LOCATION,
                format!("{}/{}", ROUTES["profile"], form_data.username),
            ))
            .finish();
    }
    let mut context = tera::Context::new();
    context.insert("error", "User already registered");
    crate::template::render_template("register.j2", session, &mut context)
}

fn validate_form(form_data: FormData) -> Result<SignUp, String> {
    lazy_static::lazy_static! {
        static ref EMAIL_REGEX: regex::Regex = regex::Regex::new(r"^[\w\-\.]+@([\w-]+\.)+\w{2,4}$").unwrap();
    }

    let username = form_data.username.unwrap_or_default();
    if username.len() < 4 {
        return Err("Username is too short, at least 4".into());
    }

    let email = form_data.email.unwrap_or_default();
    if !EMAIL_REGEX.is_match(email.as_str()) {
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
