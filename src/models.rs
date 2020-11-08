use sqlx::MySqlPool;

use serenity::{
    http::Http,
    model::{channel::Embed as SerenityEmbed, id::ChannelId, webhook::Webhook},
};

use log::warn;

use serenity::builder::CreateEmbed;
use std::time::{SystemTime, UNIX_EPOCH};

struct Embed {
    title: String,
    description: String,
    image_url: Option<String>,
    thumbnail_url: Option<String>,
    footer: String,
    footer_icon: Option<String>,
    color: u32,
}

impl Embed {
    pub async fn from_id(pool: &MySqlPool, id: u32) -> Self {
        sqlx::query_as_unchecked!(
            Self,
            "
SELECT
    title,
    description,
    image_url,
    thumbnail_url,
    footer,
    footer_icon,
    color
FROM
    embeds
WHERE
    embeds.`id` = ?
            ",
            id
        )
        .fetch_one(pool)
        .await
        .unwrap()
    }
}

impl Into<CreateEmbed> for Embed {
    fn into(self) -> CreateEmbed {
        let mut c = CreateEmbed::default();

        c.title(&self.title)
            .description(&self.description)
            .color(self.color)
            .footer(|f| {
                f.text(&self.footer);

                if let Some(footer_icon) = &self.footer_icon {
                    f.icon_url(footer_icon);
                }

                f
            });

        if let Some(image_url) = &self.image_url {
            c.image(image_url);
        }

        if let Some(thumbnail_url) = &self.thumbnail_url {
            c.thumbnail(thumbnail_url);
        }

        c
    }
}

#[derive(Debug)]
pub struct Reminder {
    id: u32,

    channel_id: u64,
    webhook_id: Option<u64>,
    webhook_token: Option<String>,

    content: String,
    tts: bool,
    embed_id: Option<u32>,
    attachment: Option<Vec<u8>>,
    attachment_name: Option<String>,

    interval: Option<u32>,
    time: u32,
}

impl Reminder {
    pub async fn fetch_reminders(pool: &MySqlPool) -> Vec<Self> {
        sqlx::query_as_unchecked!(
            Reminder,
            "
SELECT
    reminders.`id` AS id,

    channels.`channel` AS channel_id,
    channels.`webhook_id` AS webhook_id,
    channels.`webhook_token` AS webhook_token,

    messages.`content` AS content,
    messages.`tts` AS tts,
    messages.`embed_id` AS embed_id,
    messages.`attachment` AS attachment,
    messages.`attachment_name` AS attachment_name,

    reminders.`interval` AS 'interval',
    reminders.`time` AS time
FROM
    reminders
INNER JOIN
    channels
ON
    reminders.channel_id = channels.id
INNER JOIN
    messages
ON
    reminders.message_id = messages.id
WHERE
    reminders.`time` < UNIX_TIMESTAMP()
        AND ( (NOT channels.`paused`) OR channels.`paused_until` < NOW())
            "
        )
        .fetch_all(pool)
        .await
        .unwrap()
    }

    async fn reset_webhook(&self, pool: &MySqlPool) {
        sqlx::query!(
            "
UPDATE channels SET webhook_id = NULL, webhook_token = NULL WHERE channel = ?
            ",
            self.channel_id
        )
        .execute(pool)
        .await;
    }

    async fn refresh(&self, pool: &MySqlPool) {
        if let Some(interval) = self.interval {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32;
            let mut updated_reminder_time = self.time;

            while updated_reminder_time < now {
                updated_reminder_time += interval;
            }

            sqlx::query!(
                "
UPDATE reminders SET `time` = ? WHERE `id` = ?
                ",
                updated_reminder_time,
                self.id
            )
            .execute(pool)
            .await;
        } else {
            sqlx::query!(
                "
DELETE FROM reminders WHERE id = ?
                ",
                self.id
            )
            .execute(pool)
            .await;
        }
    }

    pub async fn send(&self, pool: &MySqlPool, http: &Http) {
        async fn send_to_channel(http: &Http, reminder: &Reminder, embed: Option<CreateEmbed>) {
            let channel = ChannelId(reminder.channel_id);

            channel
                .send_message(&http, |m| {
                    m.content(&reminder.content).tts(reminder.tts);

                    if let (Some(attachment), Some(name)) =
                        (&reminder.attachment, &reminder.attachment_name)
                    {
                        m.add_file((attachment as &[u8], name.as_str()));
                    }

                    if let Some(embed) = embed {
                        m.set_embed(embed);
                    }

                    m
                })
                .await;
        }

        async fn send_to_webhook(
            http: &Http,
            reminder: &Reminder,
            webhook: Webhook,
            embed: Option<CreateEmbed>,
        ) {
            webhook
                .execute(&http, false, |w| {
                    w.content(&reminder.content).tts(reminder.tts);

                    if let Some(embed) = embed {
                        w.embeds(vec![SerenityEmbed::fake(|c| {
                            *c = embed;
                            c
                        })]);
                    }

                    w
                })
                .await;
        }

        let embed = if let Some(id) = self.embed_id {
            Some(Embed::from_id(&pool.clone(), id).await.into())
        } else {
            None
        };

        if let (Some(webhook_id), Some(webhook_token)) = (self.webhook_id, &self.webhook_token) {
            let webhook_res = http.get_webhook_with_token(webhook_id, webhook_token).await;

            if let Ok(webhook) = webhook_res {
                send_to_webhook(http, &self, webhook, embed).await;
            } else {
                warn!("Webhook vanished: {:?}", webhook_res);

                self.reset_webhook(&pool.clone()).await;
                send_to_channel(http, &self, embed).await;
            }
        } else {
            send_to_channel(http, &self, embed).await;
        }

        self.refresh(pool).await;
    }
}
