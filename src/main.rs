use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::web::App;

mod users;
mod utils;
mod web;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(
            |_| "axum_login=debug,tower_sessions=debug,sqlx=warn,tower_http=debug".into(),
        )))
        .with(tracing_subscriber::fmt::layer())
        .try_init()?;

    App::from_config(&web::AppConfig {
        db_url: std::env::var("DATABASE_URL")?.to_owned(),
        listen_str: "0.0.0.0:8000".to_owned(),
    })
    .await?
    .serve()
    .await
}
