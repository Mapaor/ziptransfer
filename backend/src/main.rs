use axum::{
    Router,
    routing::{get, post, put},
};
use sqlx::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use tower_http::trace::{
    DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};

use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;

mod auth;
mod maintenance;
mod signaling;
mod transfer;

use axum::extract::ws::Message;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub signaling: Arc<DashMap<String, mpsc::UnboundedSender<Message>>>,
}

async fn root_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> axum::Json<serde_json::Value> {
    let db_status = match sqlx::query("SELECT 1").execute(&state.db).await {
        Ok(_) => "connected",
        Err(error) => {
            tracing::error!(error = %error, "Database health check failed");
            "error"
        }
    };

    axum::Json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "server_status": "online",
        "database_status": db_status
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Initialize SQLite connection
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:///app/data/sqlite.db".to_string());

    // Extract path and ensure DB file exists
    let db_path = db_url.trim_start_matches("sqlite://");
    tracing::info!(database_path = %db_path, "Initializing database");
    if let Some(parent) = std::path::Path::new(db_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    if !std::path::Path::new(db_path).exists() {
        std::fs::File::create(db_path)?;
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;
    tracing::info!("Database connection established");

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Database migrations completed");

    // Start background maintenance loop
    maintenance::start_cleanup_task(pool.clone());

    let state = AppState {
        db: pool,
        signaling: Arc::new(DashMap::new()),
    };

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(|| async { axum::http::StatusCode::OK }))
        .route("/api/health", get(|| async { "OK" }))
        .route("/api/user", get(auth::get_user))
        .route("/api/user/settings", put(auth::put_user_settings))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route(
            "/api/auth/passwordreset/request",
            post(auth::passwordreset_request),
        )
        .route("/api/auth/passwordreset/do", post(auth::passwordreset_do))
        .route("/api/signaling", get(signaling::ws_handler))
        .merge(transfer::router())
        .layer(axum::extract::DefaultBodyLimit::disable())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    tracing::info_span!(
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                    )
                })
                .on_request(DefaultOnRequest::new().level(tracing::Level::INFO))
                .on_response(DefaultOnResponse::new().level(tracing::Level::INFO))
                .on_failure(DefaultOnFailure::new().level(tracing::Level::ERROR)),
        )
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "9000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    tracing::info!(listen_address = %addr, "Starting API server");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
