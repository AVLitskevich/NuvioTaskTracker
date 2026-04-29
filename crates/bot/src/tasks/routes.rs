use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::auth::AuthenticatedUser;
use nuvio_domain::{CreateTaskRequest, TaskResponse};
use nuvio_ai::ai_service;
use crate::AppState;

#[derive(Deserialize)]
pub struct UpdateStatusRequest {
    pub status: String,
}

#[derive(Deserialize)]
pub struct AiTaskRequest {
    pub prompt: String,
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", post(create_task))
        .route("/", get(list_tasks))
        .route("/ai", post(create_task_ai))
        .route("/:id", get(get_task))
        .route("/:id/status", patch(update_task_status))
        .route("/:id", delete(delete_task))
}

pub async fn create_task_ai(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Json(body): Json<AiTaskRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let user_id = Uuid::from_u64_pair(0, user.0 as u64);
    let result = ai_service(&state.db, user_id, &body.prompt)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({"message": result})))
}

pub async fn create_task(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Json(body): Json<CreateTaskRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let task = sqlx::query_as::<_, TaskResponse>(
        "INSERT INTO tasks (user_id, idempotency_key, title, description, due_date) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (user_id, idempotency_key) DO UPDATE \
         SET title = EXCLUDED.title, description = EXCLUDED.description, due_date = EXCLUDED.due_date \
         RETURNING *",
    )
    .bind(user.0)
    .bind(&body.idempotency_key)
    .bind(&body.title)
    .bind(&body.description)
    .bind(body.due_date)
    .fetch_one(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(task)))
}

pub async fn list_tasks(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
) -> Result<impl IntoResponse, StatusCode> {
    let tasks = sqlx::query_as::<_, TaskResponse>(
        "SELECT * FROM tasks WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user.0)
    .fetch_all(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(tasks))
}

pub async fn get_task(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    let task = sqlx::query_as::<_, TaskResponse>(
        "SELECT * FROM tasks WHERE id = $1 AND user_id = $2",
    )
    .bind(id)
    .bind(user.0)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(task))
}

pub async fn update_task_status(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateStatusRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let task = sqlx::query_as::<_, TaskResponse>(
        "UPDATE tasks SET status = $1 WHERE id = $2 AND user_id = $3 RETURNING *",
    )
    .bind(&body.status)
    .bind(id)
    .bind(user.0)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(task))
}

pub async fn delete_task(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, StatusCode> {
    sqlx::query("DELETE FROM tasks WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user.0)
        .execute(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}
