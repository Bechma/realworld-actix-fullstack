#[derive(Debug)]
pub enum ConduitError {
    DatabaseError(sqlx::Error),
    TemplateError(tera::Error),
}

impl actix_web::error::ResponseError for ConduitError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            Self::DatabaseError(_) => actix_web::http::StatusCode::SEE_OTHER,
            Self::TemplateError(_) => actix_web::http::StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        match self {
            ConduitError::DatabaseError(err) => {
                eprintln!("{err:?}");
                actix_web::HttpResponse::build(actix_web::http::StatusCode::SEE_OTHER)
                    .insert_header((
                        actix_web::http::header::LOCATION,
                        actix_web::http::header::HeaderValue::from_static("error"),
                    ))
                    .finish()
            }
            ConduitError::TemplateError(te) => {
                eprintln!("{te:?}");
                actix_web::HttpResponse::build(actix_web::http::StatusCode::SERVICE_UNAVAILABLE)
                    .body("<h1>Please, try again later</h1>")
            }
        }
    }
}

impl std::fmt::Display for ConduitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConduitError::DatabaseError(e) => write!(f, "database error: {e}"),
            ConduitError::TemplateError(e) => write!(f, "cannot parse template: {e}"),
        }
    }
}

impl From<sqlx::Error> for ConduitError {
    fn from(value: sqlx::Error) -> Self {
        Self::DatabaseError(value)
    }
}

impl From<tera::Error> for ConduitError {
    fn from(value: tera::Error) -> Self {
        Self::TemplateError(value)
    }
}
