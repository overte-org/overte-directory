use crate::utils::HtmlTemplate;
use askama::Template;
use axum::Router;
use axum::response::IntoResponse;
use axum::routing::get;

use crate::web::app::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/profile", get(self::get::profile))
}

#[derive(Template)]
#[template(path = "profile.html")]
struct ProfileTemplate {}

mod get {

    use super::*;

    pub async fn profile() -> impl IntoResponse {
        HtmlTemplate(ProfileTemplate {})
    }
}
