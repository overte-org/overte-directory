use crate::{web::app::AppState, web::utils::HtmlTemplate};
use askama::Template;
use axum::{
    Form, Router,
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
};
use axum_messages::Messages;
use serde::Deserialize;

use crate::users::{AuthSession, Credentials};

#[derive(Template)]
#[template(path = "login.html")]
pub struct LoginTemplate {
    next: Option<String>,
}

#[derive(Template)]
#[template(path = "register.html")]
pub struct RegisterTemplate {}

// This allows us to extract the "next" field from the query string. We use this
// to redirect after log in.
#[derive(Debug, Deserialize)]
pub struct NextUrl {
    next: Option<String>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(self::post::login))
        .route("/login", get(self::get::login))
        .route("/logout", get(self::get::logout))
        .route("/register", post(self::post::register))
        .route("/register", get(self::get::register))
}

mod post {
    use axum::extract::State;

    use super::*;

    pub async fn login(
        mut auth_session: AuthSession,
        messages: Messages,
        Form(creds): Form<Credentials>,
    ) -> impl IntoResponse {
        let user = match auth_session.authenticate(creds.clone()).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                messages.error("Invalid credentials");

                let mut login_url = "/login".to_string();
                if let Some(next) = creds.next {
                    login_url = format!("{login_url}?next={next}");
                };

                return Redirect::to(&login_url).into_response();
            }
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        if auth_session.login(&user).await.is_err() {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        messages.success(format!("Successfully logged in as {}", user.username));

        if let Some(ref next) = creds.next {
            Redirect::to(next)
        } else {
            Redirect::to("/")
        }
        .into_response()
    }

    // FIXME: this is a temporary function
    pub async fn register(
        State(state): State<AppState>,
        messages: Messages,
        Form(creds): Form<Credentials>,
    ) -> impl IntoResponse {
        let pwhash = password_auth::generate_hash(creds.password);
        match sqlx::query!(
            "INSERT INTO users(username, password_hash) VALUES ($1, $2)",
            creds.username,
            pwhash
        )
        .execute(&state.db)
        .await
        {
            Ok(_) => {}
            Err(_) => {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        };
        messages.success("Registered :3");
        Redirect::to("/login").into_response()
    }
}

mod get {
    use super::*;

    pub async fn login(Query(NextUrl { next }): Query<NextUrl>) -> impl IntoResponse {
        HtmlTemplate(LoginTemplate { next })
    }

    pub async fn register() -> impl IntoResponse {
        HtmlTemplate(RegisterTemplate {})
    }

    pub async fn logout(mut auth_session: AuthSession) -> impl IntoResponse {
        match auth_session.logout().await {
            Ok(_) => Redirect::to("/login").into_response(),
            Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
