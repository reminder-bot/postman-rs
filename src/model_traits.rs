use crate::models::{Message, Embed, Reminder};
use crate::DISCORD_TOKEN;
use diesel::mysql::MysqlConnection;
use reqwest::Client;
use crate::diesel::prelude::*;

use serde::{Serialize};

#[derive(Serialize)]
pub struct SendableMessage {
    #[serde(skip)]
    url: String,
    #[serde(skip)]
    authorization: Option<String>,

    content: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    embed: Option<Embed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<Vec<Embed>>,

    avatar_url: Option<String>,
    username: Option<String>,
}

impl Default for SendableMessage {
    fn default() -> SendableMessage {
        SendableMessage {
            url: String::new(),
            authorization: None,
            content: String::new(),
            embed: None,
            embeds: None,
            avatar_url: None,
            username: Some(String::from("Reminder")),
        }
    }
}

impl SendableMessage {
    pub async fn send(&self, client: &Client) -> Result<(), Box<dyn std::error::Error>> {

        match &self.authorization {
            Some(auth) => {
                client.post(&self.url)
                    .body(serde_json::to_string(self)?)
                    .header("Content-Type", "application/json")
                    .header("Authorization", format!("Bot {}", auth))
            },

            None => {
                client.post(&self.url)
                    .body(serde_json::to_string(self)?)
                    .header("Content-Type", "application/json")
            }
        }.send().await?;

        Ok(())
    }
}

pub trait ReminderContent {
    fn create_sendable(&self, connection: &MysqlConnection) -> SendableMessage;
}

impl ReminderContent for Reminder {

    fn create_sendable(&self, connection: &MysqlConnection) -> SendableMessage {
        let message;
        let mut embed_handle: Option<Embed> = None;

        {
            use crate::schema::messages::dsl::*;

            // safe to unwrap- always exists under ref integrity
            message = messages.find(self.message_id)
                .load::<Message>(connection)
                .expect("Failed to query for reminder's message.")
                .pop().unwrap();

        }

        {
            use crate::schema::embeds::dsl::*;

            if let Some(message_embed_id) = message.embed_id {
                embed_handle = embeds.find(message_embed_id)
                    .load::<Embed>(connection)
                    .expect("Failed to query for reminder's message's embed.")
                    .pop();
            }
        }

        if self.is_going_to_webhook() {
            let mut embeds_vector: Option<Vec<Embed>> = None;

            if let Some(embedded_content) = embed_handle {
                embeds_vector = Some(vec![embedded_content]);
            }

            SendableMessage { url: self.get_url(), authorization: self.get_authorization(), content: message.content, embeds: embeds_vector, embed: None, avatar_url: Some(self.avatar.clone()), username: Some(self.username.clone()) }
        }
        else {
            SendableMessage { url: self.get_url(), authorization: self.get_authorization(), content: message.content, embed: embed_handle, ..Default::default() }
        }
    }
}

pub trait ApiCommunicable {
    fn is_going_to_webhook(&self) -> bool;

    fn get_url(&self) -> String;

    fn get_authorization(&self) -> Option<String>;
}

impl ApiCommunicable for Reminder {

    fn is_going_to_webhook(&self) -> bool {
        self.webhook.is_some()
    }

    fn get_url(&self) -> String {

        match &self.webhook {
            Some(url) => {
                url.to_string()
            },

            None => {
                format!("https://discordapp.com/api/v6/channels/{}/messages", self.channel)
            }
        }
    }

    fn get_authorization(&self) -> Option<String> {
        if self.is_going_to_webhook() {
            None
        }
        else {
            Some(DISCORD_TOKEN.to_string())
        }
    }

}
