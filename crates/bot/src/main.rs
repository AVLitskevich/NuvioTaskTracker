mod auth;
mod tasks;
mod bot_dispatcher;

use axum::{routing::post, extract::State, response::IntoResponse, Json, Router};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use teloxide::Bot;
use teloxide::types::Update;
use tokio::net::TcpListener;

pub struct AppState {
    pub db: sqlx::PgPool,
    pub bot: Bot,
}

async fn webhook_handler(
    State(state): State<Arc<AppState>>,
    Json(update): Json<Update>,
) -> impl IntoResponse {
    if let Err(e) = crate::bot_dispatcher::handle_update(state, update).await {
        tracing::error!("Error handling update: {:?}", e);
    }
    "ok"
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let bot_token = std::env::var("BOT_TOKEN").expect("BOT_TOKEN must be set");
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());

    let db = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    let bot = Bot::new(bot_token);
    let state = Arc::new(AppState { db, bot });

    let protected_tasks = tasks::routes::router().layer(axum::middleware::from_fn_with_state(
        state.clone(),
        auth::require_auth,
    ));

    let app = Router::new()
        .route("/api/webhook", post(webhook_handler))
        .nest("/tasks", protected_tasks)
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind TCP listener");

    tracing::info!("Listening on {addr}");
    axum::serve(listener, app)
        .await
        .expect("Server error");
}
