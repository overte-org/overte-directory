use axum::{
    Router,
    body::Body,
    http::{Request, Response},
    middleware::{Next, from_fn},
};

#[cfg(not(debug_assertions))]
use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get};
#[cfg(not(debug_assertions))]
use include_dir::{Dir, include_dir};

#[cfg(debug_assertions)]
use tower_http::services::ServeDir;

use crate::web::app::AppState;

#[cfg(not(debug_assertions))]
const ASSETS: Dir = include_dir!("assets");

pub fn router() -> Router<AppState> {
    let router = Router::new();

    #[cfg(debug_assertions)]
    let router = router.nest_service("/assets", ServeDir::new("assets"));

    #[cfg(not(debug_assertions))]
    let router = router.route("/assets", get(self::get::assets));

    router.layer(from_fn(guess_content_type))
}

async fn guess_content_type(request: Request<Body>, next: Next) -> Response<Body> {
    let uri = request.uri().to_owned();
    let path = uri.path();

    // Extract the extension before awaiting to avoid holding a borrow across await points.
    let extension = path.rsplit('.').next().map(str::to_ascii_lowercase);

    let mut response = next.run(request).await;

    if response.status().is_success() {
        let content_type = {
            let guessed = mime_guess::from_path(path).first_raw();
            match (guessed, extension.as_deref()) {
                (Some(mime), _) => mime,
                (None, Some(ext)) => {
                    tracing::warn!(%path, %ext, "Unknown MIME type; defaulting to application/octet-stream");
                    "application/octet-stream"
                }
                (None, None) => "unknown",
            }
        };

        if let Ok(content_type) = content_type.parse() {
            response.headers_mut().insert("Content-Type", content_type);
        }
    }

    response
}

#[cfg(not(debug_assertions))]
mod get {
    use super::*;

    pub async fn assets(Path(path): Path<String>) -> impl IntoResponse {
        let path = path.trim_start_matches('/');

        match ASSETS.get_file(path) {
            None => (StatusCode::NOT_FOUND, "404 Not Found").into_response(),
            Some(file) => file.contents().into_response(),
        }
    }
}
