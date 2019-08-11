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
    let req_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let pool = threadpool::ThreadPool::new(threads);

    loop {
        let mut my = mysql_conn.get_conn().unwrap().unwrap();
        let q = my.query("SELECT id, message, channel, time, position, webhook, username, avatar, embed, enabled, UNIX_TIMESTAMP() FROM reminders WHERE time < UNIX_TIMESTAMP() AND time >= 0").unwrap();

        for res in q {
            let (id, mut message, channel, mut time, position, webhook, username, avatar, color, enabled, seconds) = mysql::from_row::<(u32, String, u64, u64, Option<u32>, Option<String>, String, String, Option<u32>, bool, u64)>(res.unwrap());

            let mut req;

            let w = if webhook.is_none() || !webhook.clone().unwrap().starts_with("https") { None } else { webhook };

            if let Some(url) = w {
                let mut m;

                if let Some(color_int) = color {
                    m = Webhook {
                        content: String::new(),
                        username: username,
                        avatar_url: avatar,
                        embeds: vec![Embed { description: message, color: color_int }]
                    };
                }
                else {
                    m = Webhook {
                        content: message,
                        username: username,
                        avatar_url: avatar,
                        embeds: vec![]
                    };
                }

                req = send(url, serde_json::to_string(&m).unwrap(), None, &req_client);
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

                req = send(format!("{}/channels/{}/messages", URL, channel), serde_json::to_string(&m).unwrap(), Some(&token), &req_client);
            }

            match position {
                Some(_) => {
                    let mut reset = false;

                    while time < seconds {
                        let mut q = mysql_conn.prep_exec(r#"
                        SELECT i.period 
                            FROM intervals i, reminders r
                            WHERE 
                                i.reminder = :id AND
                                i.position = r.position MOD (
                                    SELECT COUNT(*) FROM intervals WHERE reminder = :id
                                )"#
                            , params!{"id" => id}).unwrap();

                        if let Some(row) = q.next() {
                            let period = mysql::from_row::<(u64)>(row.unwrap());
                            time += period;
                            
                            mysql_conn.prep_exec("UPDATE reminders SET position = (position + 1) MOD (SELECT COUNT(*) FROM intervals WHERE reminder = :id), time = :t WHERE id = :id", params!{"t" => time, "id" => id}).unwrap();
                        }
                        else if !reset {
                            mysql_conn.prep_exec("UPDATE reminders SET position = 0 WHERE id = :id", params!{"id" => &id}).unwrap();

                            reset = true;
                        }
                        else {
                            mysql_conn.prep_exec("DELETE FROM reminders WHERE id = :id OR time < 0", params!{"id" => &id}).unwrap();

                            break;
                        }
                    }
                },

                None => {
                    mysql_conn.prep_exec("DELETE FROM reminders WHERE id = :id OR time < 0", params!{"id" => &id}).unwrap();
                },
            }

            if enabled {
                let mut c = mysql_conn.clone();
                pool.execute(move || {
                    match req.send() {
                        Err(e) => {
                            println!("{:?}", e);
                            println!("{} {}", id, channel);
                        },

                        Ok(res) => {
                            let status: u16 = res.status().as_u16();

                            if status > 299 && status != 429 {
                                c.prep_exec("DELETE FROM reminders WHERE id = :id OR time < 0", params!{"id" => &id}).unwrap();
                            }
                        }
                    }
                });
            }
        }

        thread::sleep(Duration::from_secs(interval));
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
