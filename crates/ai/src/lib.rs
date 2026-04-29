pub mod tools;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;

pub struct GeminiClient {
    client: Client,
    api_key: String,
}

impl GeminiClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn chat_complete(&self, prompt: &str, tools: Vec<Value>) -> Result<Value, String> {
        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:runFunction?key={}", self.api_key);
        
        let payload = json!({
            "contents": [{ "parts": [{ "text": prompt }] }],
            "tools": [{ "function_declarations": tools }]
        });

        let response = self.client.post(url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let body: Value = response.json().await.map_err(|e| e.to_string())?;
        Ok(body)
    }
}

pub async fn ai_service(
    pool: &PgPool,
    user_id: uuid::Uuid,
    prompt: &str,
) -> Result<String, String> {
    let api_key = std::env::var("GEMINI_API_KEY").map_err(|_| "GEMINI_API_KEY not set".to_string())?;
    let gemini = GeminiClient::new(api_key);
    
    let tools = tools::get_tools();
    
    let response = gemini.chat_complete(prompt, tools).await?;
    
    // Check if the response contains a function call
    if let Some(function_call) = response["candidates"][0]["content"]["parts"][0]["functionCall"].as_object() {
        let name = function_call["name"].as_str().unwrap_or("");
        let args = function_call["args"].clone();
        
        if name == "add_task" {
            return tools::handle_add_task(pool, user_id, args).await;
        }
    }
    
    Ok("No action taken".to_string())
}
