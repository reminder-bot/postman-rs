#[macro_use] extern crate mysql;

extern crate dotenv;
extern crate reqwest;
extern crate threadpool;

use std::collections::HashMap;
use std::env;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH, Duration};


fn main() {
    dotenv::dotenv().ok();

    let token = env::var("DISCORD_TOKEN").unwrap();
    let sql_url = env::var("SQL_URL").unwrap();
    let interval = env::var("INTERVAL").unwrap().parse::<u64>().unwrap();
    let threads = env::var("THREADS").unwrap().parse::<usize>().unwrap();

    const URL: &str = "https://discordapp.com/api/v6";

    let mysql_conn = mysql::Pool::new(sql_url).unwrap();
    let req_client = reqwest::Client::new();
    let pool = threadpool::ThreadPool::new(threads);

    loop {
        let start = SystemTime::now();
        let since_epoch = start.duration_since(UNIX_EPOCH).expect("Time went in reverse?????");
        let seconds = since_epoch.as_secs();

        let q = mysql_conn.prep_exec("SELECT id, message, channel, time, `interval`, webhook FROM reminders WHERE time < :t", params!{"t" => seconds}).unwrap();

        for res in q {
            let (id, message, channel, mut time, interval, webhook) = mysql::from_row::<(u32, String, u64, u64, Option<u32>, Option<String>)>(res.unwrap());

            let mut req;

            if let Some(url) = webhook {
                let mut m = HashMap::new();
                m.insert("content", message);
                m.insert("username", String::from("Reminder"));
                m.insert("avatar_url", String::from("https://raw.githubusercontent.com/reminder-bot/logos/master/Remind_Me_Bot_Logo_PPic.jpg"));

                req = send(url, &m, &token, &req_client);
            }
            else {
                let mut m = HashMap::new();
                m.insert("content", message);

                req = send(format!("{}/channels/{}/messages", URL, channel), &m, &token, &req_client);
            }

            let c = mysql_conn.clone();
            let t = seconds;
            pool.execute(move || {
                match req.send() {
                    Err(_) => {},

                    Ok(_) => {
                        if let Some(interval_e) = interval {
                            while time < t {
                                time += interval_e as u64;
                            }
                            let _ = c.prep_exec("UPDATE reminders SET time = :t WHERE id = :id", params!{"t" => time, "id" => id});
                        }
                        else {
                            let _ = c.prep_exec("DELETE FROM reminders WHERE id = :id", params!{"id" => id});
                        }
                    }
                }
            });
        }

        thread::sleep(Duration::from_secs(interval));
    }
}

fn send(url: String, m: &HashMap<&str, String>, token: &str, client: &reqwest::Client) -> reqwest::RequestBuilder {
    client.post(&url)
        .json(m)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bot {}", token))
}
