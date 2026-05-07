use askama::Template;
use axum::Router;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{extract::State, http::StatusCode};

use crate::users::AuthSession;
use crate::web::app::AppState;
use crate::web::utils::HtmlTemplate;

pub fn router() -> Router<AppState> {
    Router::new().route("/profile", get(self::get::profile))
}

struct UserProfile {
    uuid: uuid::Uuid,
    username: String,
    display_name: Option<String>,
    created_time: time::UtcDateTime,
}

#[derive(Template)]
#[template(path = "profile.html")]
struct ProfileTemplate {
    user_data: UserProfile,
}

mod get {

    use super::*;

    pub async fn profile(
        State(state): State<AppState>,
        auth_session: AuthSession,
    ) -> impl IntoResponse {
        let user = &auth_session.user.unwrap();
        let user_data = match sqlx::query_as!(
            UserProfile,
            "SELECT uuid, username, display_name, created_time FROM users WHERE id = $1",
            user.id
        )
        .fetch_one(&state.db)
        .await
        {
            Ok(r) => r,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "ASDF").into_response(),
        };
        HtmlTemplate(ProfileTemplate { user_data }).into_response()
    }
}
