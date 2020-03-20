extern crate diesel;
extern crate postman;
extern crate dotenv;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

use std::env;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use self::postman::*;
use self::models::*;
use self::diesel::prelude::*;
use self::model_traits::{ReminderContent};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let refresh_interval = env::var("INTERVAL").unwrap().parse::<u64>().unwrap();

    let connection = establish_connection();

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

            reminder.create_sendable(&connection).send(&reqwest_client).await?;

            if let Some(reminder_interval) = reminder.interval {
                let mut reminder_time = reminder.time;
                while reminder_time < current_time as u32 {
                    reminder_time += reminder_interval;
                }

                diesel::update(reminders.find(reminder.id))
                    .set(time.eq(reminder_time))
                    .execute(&connection)
                    .expect("Failed to update time of interval.");
            }

            else {
                diesel::delete(reminders.find(reminder.id))
                    .execute(&connection)
                    .expect("Failed to delete expired reminder.");
            }

        }

        thread::sleep(Duration::from_secs(refresh_interval));
    }
}
