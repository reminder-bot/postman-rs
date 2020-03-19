extern crate diesel;
extern crate postman;
extern crate dotenv;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate serde_derive;

use std::env;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde_derive::{Deserialize, Serialize};

use self::postman::*;
use self::models::*;
use self::diesel::prelude::*;

/*
#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    content: String,
    embed: Option<Embed>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Webhook {
    content: String,
    username: String,
    avatar_url: String,
    embeds: Vec<Embed>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Embed {
    title: String,
    description: String,
    color: u32,
}
*/

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    dotenv::dotenv().ok();

    let token = env::var("DISCORD_TOKEN").unwrap();
    let refresh_interval = env::var("INTERVAL").unwrap().parse::<u64>().unwrap();
    let threads = env::var("THREADS").unwrap().parse::<usize>().unwrap();

    let connection = establish_connection();

    const URL: &str = "https://discordapp.com/api/v6";

    let reqwest_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to make a reqwest client");

    use postman::schema::reminders::dsl::*;

    loop {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time has reversed.").as_secs();

        let results = reminders.filter(time.le(current_time as u32))
            .load::<Reminder>(&connection)
            .expect("Error loading reminders.");

        for reminder in results {

            // Sending straight to webhook
            if let Some(webhook_url) = reminder.webhook {

            }

            // Sending to channel
            else {

            }

        }

        thread::sleep(Duration::from_secs(refresh_interval));
    }
}

fn send(url: String, m: String, token: Option<&str>, client: &reqwest::Client) -> reqwest::RequestBuilder {
    match token {
        Some(t) => {
            client.post(&url)
                .body(m)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bot {}", t))
        }

        None => {
            client.post(&url)
                .body(m)
                .header("Content-Type", "application/json")
        }
    }
}
