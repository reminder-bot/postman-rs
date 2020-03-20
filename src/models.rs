use crate::schema::{embeds, messages, reminders};

#[derive(Identifiable, Queryable, Serialize)]
#[table_name = "embeds"]
pub struct Embed {
    #[serde(skip)]
    pub id: u32,

    pub title: String,
    pub description: String,
    pub color: u32,
}

#[derive(Identifiable, Queryable)]
#[table_name = "messages"]
pub struct Message {
    pub id: u32,

    pub content: String,
    pub embed_id: Option<u32>,
}

#[derive(Identifiable, Queryable)]
#[table_name = "reminders"]
pub struct Reminder {
    pub id: u32,
    pub uid: String,

    pub message_id: u32,

    pub channel: u64,
    pub webhook: Option<String>,

    pub time: u32,
    pub interval: Option<u32>,
    pub enabled: bool,

    pub avatar: String,
    pub username: String,

    pub method: Option<String>,
}
