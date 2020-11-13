use sqlx::MySqlPool;

use serenity::{
    builder::CreateEmbed,
    http::Http,
    model::{channel::Embed as SerenityEmbed, id::ChannelId, webhook::Webhook},
    Result,
};

use log::{error, info, warn};

use sqlx::types::chrono::{NaiveDateTime, Utc};
use std::time::{SystemTime, UNIX_EPOCH};

struct Embed {
    inner: EmbedInner,
    fields: Vec<EmbedField>,
}

struct EmbedInner {
    title: String,
    description: String,
    image_url: Option<String>,
    thumbnail_url: Option<String>,
    footer: String,
    footer_icon: Option<String>,
    color: u32,
}

struct EmbedField {
    title: String,
    value: String,
    inline: bool,
}

impl Embed {
    pub async fn from_id(pool: &MySqlPool, id: u32) -> Self {
        let inner = sqlx::query_as_unchecked!(
            EmbedInner,
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
        .fetch_one(&pool.clone())
        .await
        .unwrap();

        let fields = sqlx::query_as_unchecked!(
            EmbedField,
            "
SELECT
    title,
    value,
    inline
FROM
    embed_fields
WHERE
    embed_id = ?
            ",
            id
        )
        .fetch_all(pool)
        .await
        .unwrap();

        Embed { inner, fields }
    }
}

impl Into<CreateEmbed> for Embed {
    fn into(self) -> CreateEmbed {
        let mut c = CreateEmbed::default();

        c.title(&self.inner.title)
            .description(&self.inner.description)
            .color(self.inner.color)
            .footer(|f| {
                f.text(&self.inner.footer);

                if let Some(footer_icon) = &self.inner.footer_icon {
                    f.icon_url(footer_icon);
                }

                f
            });

        for field in &self.fields {
            c.field(&field.title, &field.value, field.inline);
        }

        if let Some(image_url) = &self.inner.image_url {
            c.image(image_url);
        }

        if let Some(thumbnail_url) = &self.inner.thumbnail_url {
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

    channel_paused: bool,
    channel_paused_until: Option<NaiveDateTime>,

    content: String,
    tts: bool,
    embed_id: Option<u32>,
    attachment: Option<Vec<u8>>,
    attachment_name: Option<String>,

    interval: Option<u32>,
    time: u32,

    avatar: Option<String>,
    username: Option<String>,
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

    channels.`paused` AS channel_paused,
    channels.`paused_until` AS channel_paused_until,

    messages.`content` AS content,
    messages.`tts` AS tts,
    messages.`embed_id` AS embed_id,
    messages.`attachment` AS attachment,
    messages.`attachment_name` AS attachment_name,

    reminders.`interval` AS 'interval',
    reminders.`time` AS time,

    reminders.`avatar` AS avatar,
    reminders.`username` AS username
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
            "
        )
        .fetch_all(pool)
        .await
        .unwrap()
    }

    async fn reset_webhook(&self, pool: &MySqlPool) {
        let _ = sqlx::query!(
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
            .await
            .expect(&format!("Could not update time on Reminder {}", self.id));
        } else {
            self.force_delete(pool).await;
        }
    }

    async fn force_delete(&self, pool: &MySqlPool) {
        sqlx::query!(
            "
DELETE FROM reminders WHERE `id` = ?
            ",
            self.id
        )
        .execute(pool)
        .await
        .expect(&format!("Could not delete Reminder {}", self.id));
    }

    pub async fn send(&self, pool: &MySqlPool, http: &Http) {
        async fn send_to_channel(
            http: &Http,
            reminder: &Reminder,
            embed: Option<CreateEmbed>,
        ) -> Result<()> {
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
                .await
                .map(|_| ())
        }

        async fn send_to_webhook(
            http: &Http,
            reminder: &Reminder,
            webhook: Webhook,
            embed: Option<CreateEmbed>,
        ) -> Result<()> {
            webhook
                .execute(&http, false, |w| {
                    w.content(&reminder.content).tts(reminder.tts);

                    if let Some(username) = &reminder.username {
                        w.username(username);
                    }

                    if let Some(avatar) = &reminder.avatar {
                        w.avatar_url(avatar);
                    }

                    if let (Some(attachment), Some(name)) =
                        (&reminder.attachment, &reminder.attachment_name)
                    {
                        w.add_file((attachment as &[u8], name.as_str()));
                    }

                    if let Some(embed) = embed {
                        w.embeds(vec![SerenityEmbed::fake(|c| {
                            *c = embed;
                            c
                        })]);
                    }

                    w
                })
                .await
                .map(|_| ())
        }

        if !(self.channel_paused
            && self
                .channel_paused_until
                .map_or(true, |inner| inner >= Utc::now().naive_local()))
        {
            let _ = sqlx::query!(
                "
UPDATE `channels` SET paused = 0, paused_until = NULL WHERE `channel` = ?
                ",
                self.channel_id
            )
            .execute(&pool.clone())
            .await;

            let embed = if let Some(id) = self.embed_id {
                Some(Embed::from_id(&pool.clone(), id).await.into())
            } else {
                None
            };

            let result = if let (Some(webhook_id), Some(webhook_token)) =
                (self.webhook_id, &self.webhook_token)
            {
                let webhook_res = http.get_webhook_with_token(webhook_id, webhook_token).await;

                if let Ok(webhook) = webhook_res {
                    send_to_webhook(http, &self, webhook, embed).await
                } else {
                    warn!("Webhook vanished: {:?}", webhook_res);

                    self.reset_webhook(&pool.clone()).await;
                    send_to_channel(http, &self, embed).await
                }
            } else {
                send_to_channel(http, &self, embed).await
            };

            match result {
                Ok(()) => {
                    self.refresh(pool).await;
                }

                Err(e) => {
                    error!("Error sending {:?}: {:?}", self, e);

                    self.force_delete(pool).await;
                }
            }
        } else {
            info!("Reminder {} is paused", self.id);

            self.refresh(pool).await;
        }
    }
}
