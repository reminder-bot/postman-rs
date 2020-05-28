use crate::models::{Message, Embed, Reminder, Channel};
use crate::DISCORD_TOKEN;
use diesel::mysql::MysqlConnection;
use reqwest::{Client, multipart};
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
    embed: Option<SendableEmbed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    embeds: Option<Vec<SendableEmbed>>,

    tts: bool,

    avatar_url: Option<String>,
    username: Option<String>,

    #[serde(skip)]
    attachment: Option<Vec<u8>>,
    #[serde(skip)]
    attachment_name: Option<String>,
}

impl Default for SendableMessage {
    fn default() -> SendableMessage {
        SendableMessage {
            url: String::new(),
            authorization: None,
            content: String::new(),
            embed: None,
            embeds: None,
            tts: false,
            avatar_url: None,
            username: Some(String::from("Reminder")),
            attachment: None,
            attachment_name: None,
        }
    }
}

impl SendableMessage {
    fn construct_multipart(&self) -> Result<multipart::Form, Box<dyn std::error::Error>> {
        let json = serde_json::to_string(self)?;

        if self.attachment.is_some() && self.attachment_name.is_some() {
            let attachment = self.attachment.clone().unwrap();
            let name = self.attachment_name.clone().unwrap();

            let form = multipart::Form::new()
                .text("payload_json", json)
                .part("file", multipart::Part::bytes(attachment).file_name(name));

            Ok(form)
        }
        else {
            let form = multipart::Form::new()
                .text("payload_json", json);

            Ok(form)
        }
    }

    pub async fn send(&self, client: &Client) -> Result<reqwest::StatusCode, Box<dyn std::error::Error>> {

        let form = self.construct_multipart()?;

        let response = match &self.authorization {
            Some(auth) => {
                client.post(&self.url)
                    .multipart(form)
                    .header("Content-Type", "multipart/form-data")
                    .header("Authorization", format!("Bot {}", auth))
            },

            None => {
                client.post(&self.url)
                    .multipart(form)
                    .header("Content-Type", "multipart/form-data")
            }
        }.send().await?;

        Ok(response.status())
    }
}

#[derive(Serialize)]
pub struct Footer {
    pub text: String,
    pub icon_url: String,
}

#[derive(Serialize)]
pub struct SendableEmbed {
    pub title: String,
    pub description: String,
    pub footer: Footer,

    pub color: u32,
}

impl SendableEmbed {
    pub fn from_embed(embed: Embed) -> Self {
        return SendableEmbed {
            title: embed.title,
            description: embed.description,
            footer: Footer {
                text: embed.footer,
                icon_url: embed.footer_icon,
            },
            color: embed.color,
        }
    }
}

pub struct ReminderDetails<'a> {
    pub channel: Channel,

    pub reminder: &'a Reminder,
}

impl<'a> ReminderDetails<'a> {
    pub fn create_from_reminder(reminder: &'a Reminder, connection: &MysqlConnection) -> ReminderDetails<'a> {
        let reminder_channel: Channel;

        {
            use crate::schema::channels::dsl::*;

            reminder_channel = channels.find(reminder.channel_id)
                .load::<Channel>(connection)
                .expect("Couldn't get reminder channel")
                .pop().expect("No reminder channel found (violated Ref Integrity)");
        }

        ReminderDetails { reminder, channel: reminder_channel }
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

        let sendable_embed_handle = embed_handle.map(|e| SendableEmbed::from_embed(e));

        if self.is_going_to_webhook() {
            let mut embeds_vector: Option<Vec<SendableEmbed>> = None;

            if let Some(embedded_content) = sendable_embed_handle {
                embeds_vector = Some(vec![embedded_content]);
            }

            SendableMessage {
                url: self.get_url(),
                authorization: self.get_authorization(),
                content: message.content,
                embeds: embeds_vector,
                embed: None,
                tts: message.tts,
                avatar_url: Some(self.reminder.avatar.clone()),
                username: Some(self.reminder.username.clone()),
                attachment: message.attachment,
                attachment_name: message.attachment_name,
            }
        }
        else {
            SendableMessage {
                url: self.get_url(),
                authorization: self.get_authorization(),
                content: message.content,
                embed: sendable_embed_handle,
                tts: message.tts,
                attachment: message.attachment,
                attachment_name: message.attachment_name,
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
        self.channel.webhook_id.is_some() && self.channel.webhook_token.is_some()
    }

    fn get_url(&self) -> String {

        if self.is_going_to_webhook() {
            let c = &self.channel;
            format!("https://discordapp.com/api/webhooks/{}/{}", c.webhook_id.as_ref().unwrap(), c.webhook_token.as_ref().unwrap())
        }
        else {
            format!("https://discordapp.com/api/v6/channels/{}/messages", self.channel.channel)
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
