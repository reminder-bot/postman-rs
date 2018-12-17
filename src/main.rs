#[macro_use] extern crate mysql;

extern crate dotenv;
extern crate reqwest;
extern crate threadpool;
extern crate serde;
extern crate serde_json;
extern crate serde_derive;

use std::env;
use std::thread;
use std::time::Duration;
use serde_derive::{Deserialize, Serialize};

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
    description: String,
    color: u32,
}

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
        pool.join();

        let mut my = mysql_conn.get_conn().unwrap().unwrap();
        let q = my.query("SELECT id, message, channel, time, `interval`, webhook, embed, UNIX_TIMESTAMP() FROM reminders WHERE time < UNIX_TIMESTAMP()").unwrap();

        for res in q {
            let (id, mut message, channel, mut time, interval, webhook, color, seconds) = mysql::from_row::<(u32, String, u64, u64, Option<u32>, Option<String>, Option<u32>, u64)>(res.unwrap());

            let mut req;

            if let Some(url) = webhook {
                let mut m;

                if let Some(color_int) = color {
                    m = Webhook {
                        content: String::new(),
                        username: String::from("Reminder"),
                        avatar_url: String::from("https://raw.githubusercontent.com/reminder-bot/logos/master/Remind_Me_Bot_Logo_PPic.jpg"),
                        embeds: vec![Embed { description: message, color: color_int }]
                    };
                }
                else {
                    m = Webhook {
                        content: message,
                        username: String::from("Reminder"),
                        avatar_url: String::from("https://raw.githubusercontent.com/reminder-bot/logos/master/Remind_Me_Bot_Logo_PPic.jpg"),
                        embeds: vec![]
                    };
                }

                req = send(url, serde_json::to_string(&m).unwrap(), &token, &req_client);
            }
            else {
                let mut m;

                if let Some(color_int) = color {
                    m = Message {
                        content: String::new(),
                        embed: Some(Embed { description: message, color: color_int }),
                    };
                }
                else {
                    m = Message {
                        content: message,
                        embed: None
                    };
                }

                req = send(format!("{}/channels/{}/messages", URL, channel), serde_json::to_string(&m).unwrap(), &token, &req_client);
            }

            let c = mysql_conn.clone();
            pool.execute(move || {
                match req.send() {
                    Err(e) => {
                        println!("{:?}", e);
                    },

                    Ok(mut r) => {
                        println!("{:?}", r);
                        println!("{:?}", r.text());

                        if let Some(interval_e) = interval {
                            while time < seconds {
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

fn send(url: String, m: String, token: &str, client: &reqwest::Client) -> reqwest::RequestBuilder {
    client.post(&url)
        .body(m)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bot {}", token))
}
