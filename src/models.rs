#[derive(Queryable)]
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

    pub method: String,
}

#[derive(Queryable)]
pub struct Message {
    pub id: u32,

    pub content: String,
    pub embed: Option<u32>,
}

#[derive(Queryable)]
pub struct Embed {
    pub id: u32,

    pub title: String,
    pub description: String,
    pub color: u32,
}
