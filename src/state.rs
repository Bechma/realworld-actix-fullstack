use actix_web::{
    http::{header::ContentType, StatusCode},
    HttpResponse,
};

pub type AppState = std::sync::Arc<AppStateStruct>;

pub struct AppStateStruct {
    templates: tera::Tera,
    routes: crate::routing::Routes,
    email_regex: regex::Regex,
}

impl AppStateStruct {
    pub fn new(template: tera::Tera) -> Self {
        Self {
            templates: template,
            routes: crate::routing::Routes::new(),
            email_regex: regex::Regex::new(r"^[\w\-\.]+@([\w-]+\.)+\w{2,4}$").expect("bad regex"),
        }
    }

    pub fn render_template(
        &self,
        template: &str,
        session: &actix_session::Session,
        context: &mut tera::Context,
    ) -> Result<HttpResponse, crate::errors::ConduitError> {
        let current = template.replace(".j2", "");
        if !context.contains_key("current") {
            context.insert("current", &current);
        }
        context.insert("routes", &self.routes);
        if let Some(username) = crate::utils::get_session_username(session) {
            context.insert("username", &username);
        }
        let body = self.templates.render(template, context)?;
        Ok(HttpResponse::build(StatusCode::OK)
            .content_type(ContentType::html())
            .body(body))
    }

    pub fn redirect_to_profile(&self, session: &actix_session::Session) -> Option<HttpResponse> {
        self.routes.redirect_to_profile(session)
    }

    pub(crate) fn route_from_enum(&self, value: &crate::routing::RoutesEnum) -> String {
        self.routes.enum_to_string(value)
    }

    pub fn apply_routes(&self) -> impl Fn(&mut actix_web::web::ServiceConfig) {
        self.routes.apply_routes()
    }

    pub fn is_email(&self, email: &str) -> bool {
        self.email_regex.is_match(email)
    }
}
