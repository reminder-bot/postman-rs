mod macros;
mod models;

use log::info;

use sqlx::MySqlPool;

use serenity::http::Http;

use dotenv::dotenv;

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();

    dotenv()?;

    let pool =
        MySqlPool::new(&env::var("DATABASE_URL").expect("Missing DATABASE_URL from environment"))
            .await
            .unwrap();

    let token = &env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not provided in environment");

    let http = Http::new_with_token(&token);

    loop {
        let reminders = models::Reminder::fetch_reminders(&pool).await;

        info!("Preparing to send {} reminders:", reminders.len());

        for reminder in reminders {
            info!("\tPreparing to send {:?}", reminder);

            reminder.send(&pool, &http).await;
        }
    }
}
