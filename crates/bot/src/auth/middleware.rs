use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::AppState;

/// Axum middleware that validates Telegram WebApp initData presented as a
/// Bearer token, upserts the user into the `users` table, and injects the
/// `i64` telegram user id into request extensions.
///
/// Returns 401 Unauthorized if any step fails.
pub async fn require_auth(
    State(state): State<Arc<AppState>>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    // 1. Read `Authorization: Bearer <initData>` header.
    let init_data = match req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
    {
        Some(token) => token.to_owned(),
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    // 2. Obtain current Unix timestamp.
    let now_unix = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs(),
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };

    // 3. Validate initData signature and freshness.
    let fields = match crate::auth::telegram::validate_init_data(&init_data, state.bot.token(), now_unix) {
        Ok(f) => f,
        Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
    };

    // 4. Parse the `user` JSON field and extract `id: i64`.
    let user_id: i64 = match fields.get("user") {
        Some(user_json) => match serde_json::from_str::<serde_json::Value>(user_json) {
            Ok(v) => match v.get("id").and_then(|id| id.as_i64()) {
                Some(id) => id,
                None => return StatusCode::UNAUTHORIZED.into_response(),
            },
            Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
        },
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    // 5. Upsert user: INSERT INTO users (id) VALUES ($1) ON CONFLICT (id) DO NOTHING.
    if sqlx::query("INSERT INTO users (id) VALUES ($1) ON CONFLICT (id) DO NOTHING")
        .bind(user_id)
        .execute(&state.db)
        .await
        .is_err()
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    // 6. Insert the user id into request extensions for downstream extractors.
    req.extensions_mut().insert(user_id);

    // 7. Pass through to the next handler.
    next.run(req).await
}
