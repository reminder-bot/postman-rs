use crate::schema::{embeds, messages, reminders, channels};
use chrono::NaiveDateTime;

#[derive(Identifiable, Queryable)]
#[table_name = "embeds"]
pub struct Embed {
    pub id: u32,

    pub title: String,
    pub description: String,
    pub footer: String,
    pub footer_icon: Option<String>,

    pub color: u32,
}

#[derive(Identifiable, Queryable)]
#[table_name = "messages"]
pub struct Message {
    pub id: u32,

    pub content: String,
    pub tts: bool,
    pub embed_id: Option<u32>,

    pub attachment: Option<Vec<u8>>,
    pub attachment_name: Option<String>
}

#[derive(Identifiable, Queryable)]
#[table_name = "reminders"]
pub struct Reminder {
    pub id: u32,
    pub uid: String,

    pub message_id: u32,

    pub channel_id: u32,

    pub time: u32,
    pub interval: Option<u32>,
    pub enabled: bool,

    pub avatar: String,
    pub username: String,

    pub method: Option<String>,
}

#[derive(Identifiable, Queryable)]
#[table_name = "channels"]
pub struct Channel {
    pub id: u32,
    pub channel: u64,

    pub nudge: i16,
    pub blacklisted: bool,

    pub name: Option<String>,

    pub webhook_id: Option<u64>,
    pub webhook_token: Option<String>,

    pub paused: bool,
    pub paused_until: Option<NaiveDateTime>,

    pub guild_id: Option<u32>,
}
