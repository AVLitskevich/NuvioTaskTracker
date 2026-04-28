use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    /// Client-generated idempotency key (UUID v4 recommended).
    pub idempotency_key: String,
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TaskResponse {
    pub id: Uuid,
    pub idempotency_key: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub due_date: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}
