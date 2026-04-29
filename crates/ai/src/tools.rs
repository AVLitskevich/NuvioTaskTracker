use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

pub fn get_tools() -> Vec<Value> {
    vec![json!({
        "name": "add_task",
        "description": "Adds a new task to the task tracker",
        "parameters": {
            "type": "OBJECT",
            "properties": {
                "title": { "type": "STRING", "description": "The title of the task" },
                "description": { "type": "STRING", "description": "The description of the task" },
                "due_date": { "type": "STRING", "description": "Optional due date (ISO 8601)" }
            },
            "required": ["title"]
        }
    })]
}

pub async fn handle_add_task(
    pool: &PgPool,
    user_id: Uuid,
    args: Value,
) -> Result<String, String> {
    let title = args["title"].as_str().ok_or("Missing title")?;
    let description = args["description"].as_str().unwrap_or("");
    let due_date: Option<time::OffsetDateTime> = args["due_date"]
        .as_str()
        .and_then(|s| time::OffsetDateTime::parse(s, &time::format_description::well_known::Iso8601::DEFAULT).ok());

    sqlx::query(
        "INSERT INTO tasks (user_id, idempotency_key, title, description, due_date) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(Uuid::new_v4().to_string())
    .bind(title)
    .bind(description)
    .bind(due_date)
    .execute(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(format!("Task '{}' added successfully", title))
}
