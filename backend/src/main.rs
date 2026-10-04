use axum::{routing::{get, post, put}, Router};
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;

use std::sync::Arc;
use dashmap::DashMap;
use tokio::sync::mpsc;

mod auth;
mod signaling;
mod transfer;
mod maintenance;

use axum::extract::ws::Message;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub signaling: Arc<DashMap<String, mpsc::UnboundedSender<Message>>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // Initialize SQLite connection
    let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:///app/data/sqlite.db".to_string());
    
    // Extract path and ensure DB file exists
    let db_path = db_url.trim_start_matches("sqlite://");
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

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Start background maintenance loop
    maintenance::start_cleanup_task(pool.clone());

    let state = AppState { 
        db: pool,
        signaling: Arc::new(DashMap::new()),
    };

    let app = Router::new()
        .route("/api/health", get(|| async { "OK" }))
        .route("/api/user", get(auth::get_user))
        .route("/api/user/settings", put(auth::put_user_settings))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/passwordreset/request", post(auth::passwordreset_request))
        .route("/api/auth/passwordreset/do", post(auth::passwordreset_do))
        .route("/api/signaling", get(signaling::ws_handler))
        .merge(transfer::router())
        .layer(axum::extract::DefaultBodyLimit::disable())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "9000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    println!("Starting server on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
