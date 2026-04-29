use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

/// Extracts the Telegram user ID previously set by auth middleware.
/// Middleware must validate initData and insert `i64` (telegram_user_id) into
/// request extensions before routes run.
pub struct AuthenticatedUser(pub i64);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for AuthenticatedUser {
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<i64>()
            .copied()
            .map(AuthenticatedUser)
            .ok_or(StatusCode::UNAUTHORIZED)
    }
}
