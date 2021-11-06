#[macro_use]
extern crate lazy_static;

mod models;
mod substitutions;

use log::{info, warn};

use sqlx::MySqlPool;

use serenity::http::Http;

use dotenv::dotenv;

use std::sync::Arc;
use std::{env, time::Duration};
use tokio::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();

    dotenv()?;

    let dry_run = env::args()
        .collect::<Vec<String>>()
        .contains(&"--dry-run".to_string());

    let interval = env::var("INTERVAL")
        .map(|inner| inner.parse::<u64>().ok())
        .ok()
        .flatten()
        .unwrap_or_else(|| {
            warn!("No interval has been provided: defaulting to 10 seconds");

            10
        });

    info!("dry-run: {}", dry_run);
    info!("interval: {}", interval);

    let pool = MySqlPool::connect(
        &env::var("DATABASE_URL").expect("Missing DATABASE_URL from environment"),
    )
    .await
    .unwrap();

    let token = &env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not provided in environment");

    let http = Http::new_with_token(&token);

    let arc = Arc::new(http);

    loop {
        let sleep_until = Instant::now() + Duration::from_secs(interval);
        let reminders = models::Reminder::fetch_reminders(&pool).await;

        if reminders.len() > 0 {
            info!("=================================================");
            info!("Preparing to send {} reminders:", reminders.len());

            for reminder in reminders {
                info!("Sending {:?}", reminder);

                if !dry_run {
                    let pool_clone = pool.clone();
                    let http_clone = arc.clone();

                    reminder.send(pool_clone, http_clone).await;
                } else {
                    info!("(( dry run; nothing sent ))");
                }
            }
        }

        tokio::time::sleep_until(sleep_until).await;
    }
}
