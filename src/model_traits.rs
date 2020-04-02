use crate::models::{Message, Embed, Reminder, Channel, User};
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

pub struct ReminderDetails<'a> {
    channel: Option<Channel>,
    user: Option<User>,

    pub reminder: &'a Reminder,
}

impl<'a> ReminderDetails<'a> {
    pub fn create_from_reminder(reminder: &'a Reminder, connection: &MysqlConnection) -> ReminderDetails<'a> {
        let mut reminder_channel: Option<Channel> = None;
        let mut reminder_user: Option<User> = None;

        if let Some(channel_id) = reminder.channel_id {
            use crate::schema::channels::dsl::*;

            reminder_channel = channels.find(channel_id)
                .load::<Channel>(connection)
                .expect("Couldn't get reminder channel")
                .pop();
        }
        else {
            use crate::schema::users::dsl::*;

            reminder_user = users.find(reminder.user_id.unwrap())
                .load::<User>(connection)
                .expect("Couldn't get reminder user")
                .pop();
        }

        ReminderDetails { reminder, channel: reminder_channel, user: reminder_user }
    }
}

pub trait ReminderContent {
    fn create_sendable(&self, connection: &MysqlConnection) -> SendableMessage;
}

impl ReminderContent for ReminderDetails<'_> {

    fn create_sendable(&self, connection: &MysqlConnection) -> SendableMessage {
        let message;
        let mut embed_handle: Option<Embed> = None;

        {
            use crate::schema::messages::dsl::*;

            // safe to unwrap- always exists under ref integrity
            message = messages.find(self.reminder.message_id)
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

            SendableMessage {
                url: self.get_url(),
                authorization: self.get_authorization(),
                content: message.content,
                embeds: embeds_vector,
                embed: None,
                avatar_url: Some(self.reminder.avatar.clone()),
                username: Some(self.reminder.username.clone())
            }
        }
        else {
            SendableMessage {
                url: self.get_url(),
                authorization: self.get_authorization(),
                content: message.content,
                embed: embed_handle,
                ..Default::default()
            }
        }
    }
}

pub trait ApiCommunicable {
    fn is_going_to_webhook(&self) -> bool;

    fn get_url(&self) -> String;

    fn get_authorization(&self) -> Option<String>;
}

impl ApiCommunicable for ReminderDetails<'_> {

    fn is_going_to_webhook(&self) -> bool {
        match &self.channel {
            Some(channel) => {
                channel.webhook_id.is_some() && channel.webhook_token.is_some()
            }

            None => {
                false
            }
        }
    }

    fn get_url(&self) -> String {

        if self.is_going_to_webhook() {
            let c = self.channel.as_ref().unwrap();
            format!("https://discordapp.com/api/webhooks/{}/{}", c.webhook_id.as_ref().unwrap(), c.webhook_token.as_ref().unwrap())
        }
        else if let Some(channel) = &self.channel {
            format!("https://discordapp.com/api/v6/channels/{}/messages", channel.channel)
        }
        else if let Some(user) = &self.user {
            format!("https://discordapp.com/api/v6/channels/{}/messages", user.dm_channel)
        }
        else {
            panic!("Reminder found with neither channel nor user specified");
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
