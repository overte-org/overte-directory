use askama::Template;
use axum::response::IntoResponse;
use axum::{Router, routing::get};
use axum_session::SessionLayer;
use axum_session_sqlx::SessionPgPool;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod utils;

use crate::utils::HtmlTemplate;
#[cfg(not(debug_assertions))]
use crate::utils::shutdown_signal;

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate {
    name: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or("debug,hyper=off".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // TODO: use envvars for config
    let pg_pool = sqlx::PgPool::connect("postgres://app:app@127.0.0.1:5432/app")
        .await
        .expect("database to connect");

    let session_config = axum_session::SessionConfig::default().with_table_name("sessions_table");
    let session_store = axum_session::SessionStore::<SessionPgPool>::new(
        Some(pg_pool.clone().into()),
        session_config,
    )
    .await
    .expect("session storage to init");

    let app = Router::new()
        .route("/", get(home))
        .route("/login", get(login_page))
        .layer(SessionLayer::new(session_store));

    // debug live_reload
    #[cfg(debug_assertions)]
    let app = {
        use notify::Watcher;
        let livereload = tower_livereload::LiveReloadLayer::new().request_predicate(
            |req: &axum::http::Request<axum::body::Body>| !req.headers().contains_key("hx-request"),
        );
        let reloader = livereload.reloader();
        let mut watcher = notify::recommended_watcher(move |_| reloader.reload()).unwrap();
        watcher
            .watch(
                std::path::Path::new("templates"),
                notify::RecursiveMode::Recursive,
            )
            .unwrap();
        tracing::info!("Reloading!");
        app.layer(livereload)
    };

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::info!("listening on {:?}", listener.local_addr().unwrap());
    let serve = axum::serve(listener, app);

    // graceful shutdown only in prod
    #[cfg(not(debug_assertions))]
    let serve = serve.with_graceful_shutdown(shutdown_signal());

    serve.await.unwrap();
}

async fn home() -> impl IntoResponse {
    HtmlTemplate(HelloTemplate {
        name: "asdf".to_string(),
    })
}

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {}

async fn login_page() -> impl IntoResponse {
    HtmlTemplate(LoginTemplate {})
}
