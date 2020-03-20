pub mod schema;
pub mod models;
pub mod model_traits;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate lazy_static;

extern crate dotenv;

use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use dotenv::dotenv;
use std::env;

lazy_static! {
    static ref DISCORD_TOKEN: String = {
        dotenv().ok();
        env::var("DISCORD_TOKEN").unwrap()
    };
}

pub fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not provided in environment or .env");

    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}
