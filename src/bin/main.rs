extern crate diesel;
extern crate postman;
extern crate dotenv;
extern crate reqwest;
extern crate serde;
extern crate serde_json;

use std::env;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use reqwest::StatusCode;

use self::postman::*;
use self::models::*;
use self::diesel::prelude::*;
use self::model_traits::{ReminderContent, ReminderDetails};
use self::postman::model_traits::ApiCommunicable;

use dotenv::dotenv;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let refresh_interval = env::var("INTERVAL").unwrap().parse::<u64>().unwrap();

    let connection = establish_connection();

    let reqwest_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("Failed to make a reqwest client");

    use postman::schema::reminders::dsl::*;
    use postman::schema::channels::dsl::*;

    loop {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time has reversed.").as_secs();

        let results = reminders.filter(time.le(current_time as u32))
            .load::<Reminder>(&connection)
            .expect("Error loading reminders.");

        for reminder_wrapper in results.iter().map(|r| { ReminderDetails::create_from_reminder(r, &connection) }) {
            let reminder = &reminder_wrapper.reminder;

            if reminder_wrapper.should_send(&connection) {
                let status_code = reminder_wrapper.create_sendable(&connection).send(&reqwest_client).await?;

                if status_code == StatusCode::NOT_FOUND {
                    if reminder_wrapper.is_going_to_webhook() {

                        let reminder_channel = reminder_wrapper.channel;

                        diesel::update(channels.find(reminder_channel.id))
                            .set((webhook_id.eq::<Option<u64>>(None), webhook_token.eq::<Option<String>>(None)))
                            .execute(&connection)
                            .expect("Failed to remove webhook token and ID from 404 reminder");
                    }
                }
                else if reminder.interval.is_some() && !status_code.is_success() {
                    diesel::delete(reminders.find(reminder.id))
                        .execute(&connection)
                        .expect("Failed to delete failing interval");
                }
            }

            if let Some(reminder_interval) = reminder.interval {
                let mut reminder_time = reminder.time;
                while reminder_time <= current_time as u32 {
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
