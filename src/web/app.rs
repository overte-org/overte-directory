use crate::{
    users::{AuthSession, Backend},
    utils::shutdown_signal,
    web::{assets, auth, profile},
};
use axum::{
    BoxError, Router,
    body::Body,
    error_handling::HandleErrorLayer,
    http::{Response, StatusCode},
    response::{Html, IntoResponse},
};
use axum_login::{AuthManagerLayerBuilder, login_required, tower_sessions::ExpiredDeletion};
use axum_session::Key;
use sqlx::PgPool;
use time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;

/// App configuration
#[derive(Debug)]
pub struct AppConfig {
    /// See: https://docs.rs/sqlx/latest/sqlx/postgres/struct.PgConnectOptions.html
    pub db_url: String,
    pub listen_str: String,
}

pub struct App {
    db: PgPool,
    listen_str: String,
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: PgPool,
}

impl App {
    pub async fn from_config(config: &AppConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let db = PgPool::connect(&config.db_url).await?;

        //sqlx::migrate!().run(&db).await?;

        Ok(Self {
            db,
            listen_str: config.listen_str.clone(),
        })
    }

    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        // Session layer.
        //
        // This uses `tower-sessions` to establish a layer that will provide the session
        // as a request extension.
        let (session_layer, deletion_task) = {
            let session_store = PostgresStore::new(self.db.clone());
            session_store.migrate().await?;

            let deletion_task = tokio::task::spawn(
                session_store
                    .clone()
                    .continuously_delete_expired(tokio::time::Duration::from_secs(60)),
            );

            // Generate a cryptographic key to sign the session cookie.
            let key = Key::generate();

            (
                SessionManagerLayer::new(session_store)
                    .with_secure(false)
                    .with_expiry(Expiry::OnInactivity(Duration::days(1)))
                    .with_signed(key),
                deletion_task,
            )
        };

        // Auth service.
        //
        // This combines the session layer with our backend to establish the auth
        // service which will provide the auth session as a request extension.
        let auth_layer = {
            let backend = Backend::new(self.db.clone());
            AuthManagerLayerBuilder::new(backend, session_layer).build()
        };

        let csrf_protection_layer = {
            async fn handle_error(error: BoxError) -> Response<Body> {
                if let Some(csrf) = error.downcast_ref::<tower_csrf::ProtectionError>() {
                    tracing::error!("csrf error occured: {}", csrf);
                    return StatusCode::FORBIDDEN.into_response();
                }

                tracing::error!("error occured: {}", error);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
            tower::ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                .layer(
                    tower_csrf::CrossOriginProtectionLayer::default()
                        .add_trusted_origin("http://127.0.0.1:8000")
                        .unwrap(),
                )
        };

        let app = Router::new() //protected::router()
            .route(
                "/",
                axum::routing::get(|auth_session: AuthSession| async {
                    Html(format!(
                        "Logged in as {} <a href=\"/logout\">Logout</a>",
                        auth_session.user.unwrap().username
                    ))
                }),
            )
            .merge(profile::router())
            .route_layer(login_required!(Backend, login_url = "/login"))
            .merge(assets::router())
            .merge(auth::router())
            .layer(axum_messages::MessagesManagerLayer)
            .layer(auth_layer)
            .layer(csrf_protection_layer)
            .with_state(AppState {
                db: self.db.clone(),
            });

        let listener = tokio::net::TcpListener::bind(&self.listen_str)
            .await
            .unwrap();

        // Ensure we use a shutdown signal to abort the deletion task.
        axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        deletion_task.abort_handle().abort();
        let _ = deletion_task.await?;

        Ok(())
    }
}
