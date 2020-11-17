mod models;

use log::info;

use sqlx::MySqlPool;

use serenity::http::Http;

use dotenv::dotenv;

use log::warn;

use std::{env, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();

    dotenv()?;

    let dry_run = env::args()
        .collect::<Vec<String>>()
        .contains(&"--dry-run".to_string());

    println!("dry-run: {}", dry_run);

    let interval = env::var("INTERVAL")
        .map(|inner| inner.parse::<u64>().ok())
        .ok()
        .flatten()
        .unwrap_or_else(|| {
            warn!("No interval has been provided: defaulting to 10 seconds");

            10
        });

    let pool =
        MySqlPool::new(&env::var("DATABASE_URL").expect("Missing DATABASE_URL from environment"))
            .await
            .unwrap();

    let token = &env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not provided in environment");

    let http = Http::new_with_token(&token);

    loop {
        let reminders = models::Reminder::fetch_reminders(&pool).await;

        if reminders.len() > 0 {
            info!("=================================================");
            info!("Preparing to send {} reminders:", reminders.len());

            for reminder in reminders {
                info!("Sending {:?}", reminder);

                if !dry_run {
                    reminder.send(&pool, &http).await;
                } else {
                    info!("(( dry run; nothing sent ))");
                }
            }

            tokio::time::delay_for(Duration::from_secs(interval)).await;
        }
    }
}
