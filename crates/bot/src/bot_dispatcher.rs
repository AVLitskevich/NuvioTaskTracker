use teloxide::prelude::*;
use teloxide::utils::command::BotCommands;
use teloxide::types::{InlineKeyboardMarkup, InlineKeyboardButton, WebAppInfo};
use std::sync::Arc;
use crate::AppState;
use uuid::Uuid;
use nuvio_ai::ai_service;
use nuvio_domain::TaskResponse;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "These commands are supported:")]
pub enum Command {
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "Show help")]
    Help,
    #[command(description = "Add a task: /add <task text>")]
    Add(String),
    #[command(description = "List all tasks")]
    List,
}

pub async fn answer(bot: Bot, msg: Message, cmd: Command, state: Arc<AppState>) -> ResponseResult<()> {
    match cmd {
        Command::Start => {
            let web_app_url = std::env::var("WEB_APP_URL").unwrap_or_else(|_| "https://example.com".to_string());
            let keyboard = InlineKeyboardMarkup::new(vec![vec![
                InlineKeyboardButton::web_app("Open Mini App", WebAppInfo { url: web_app_url.parse().unwrap() })
            ]]);
            
            bot.send_message(msg.chat.id, "Hello! I'm Nuvio Bot. Use the buttons below to open the app or use commands like /add and /list.")
                .reply_markup(keyboard)
                .await?;
        }
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string()).await?;
        }
        Command::Add(text) => {
            if let Some(user) = msg.from() {
                let user_id = Uuid::from_u64_pair(0, user.id.0 as u64);
                match ai_service(&state.db, user_id, &text).await {
                    Ok(response) => {
                        bot.send_message(msg.chat.id, format!("AI Service: {}", response)).await?;
                    }
                    Err(e) => {
                        bot.send_message(msg.chat.id, format!("Error: {}", e)).await?;
                    }
                }
            }
        }
        Command::List => {
            if let Some(user) = msg.from() {
                let user_id = user.id.0 as i64;
                let tasks = sqlx::query_as::<_, TaskResponse>(
                    "SELECT * FROM tasks WHERE user_id = $1 ORDER BY created_at DESC",
                )
                .bind(user_id)
                .fetch_all(&state.db)
                .await;

                match tasks {
                    Ok(tasks) => {
                        if tasks.is_empty() {
                            bot.send_message(msg.chat.id, "You have no tasks.").await?;
                        } else {
                            let list = tasks.iter().map(|t| format!("- {}", t.title)).collect::<Vec<_>>().join("\n");
                            bot.send_message(msg.chat.id, format!("Your tasks:\n{}", list)).await?;
                        }
                    }
                    Err(e) => {
                        bot.send_message(msg.chat.id, format!("Database error: {}", e)).await?;
                    }
                }
            }
        }
    };
    Ok(())
}

pub async fn handle_update(state: Arc<AppState>, update: Update) -> Result<(), Box<dyn std::error::Error>> {
    use teloxide::dispatching::{HandlerExt, UpdateFilterExt};

    let bot = state.bot.clone();
    let handler = Update::filter_message()
        .filter_command::<Command>()
        .endpoint(answer);

    let _ = handler.dispatch(dptree::deps![bot, state, update]).await;

    Ok(())
}
