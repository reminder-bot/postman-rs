use sqlx::MySqlPool;

use serenity::{
    http::Http,
    model::{id::ChannelId, webhook::Webhook},
};

use crate::json;

use log::warn;

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct Reminder {
    id: u32,

    channel_id: u64,
    webhook_id: Option<u64>,
    webhook_token: Option<String>,

    content: String,

    interval: Option<u32>,
    time: u32,
}

impl Reminder {
    pub async fn fetch_reminders(pool: &MySqlPool) -> Vec<Self> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

        sqlx::query_as!(
            Reminder,
            "
SELECT
    reminders.`id` AS id,

    channels.`channel` AS channel_id,
    channels.`webhook_id` AS webhook_id,
    channels.`webhook_token` AS webhook_token,

    messages.`content` AS content,

    reminders.`interval` AS 'interval',
    reminders.`time` AS time
FROM
    reminders
INNER JOIN
    channels
ON
    reminders.id = channels.id
INNER JOIN
    messages
ON
    reminders.message_id = messages.id
WHERE
    reminders.`time` < ?
            ",
            now.as_secs()
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
        async fn send_to_channel(http: &Http, reminder: &Reminder) {
            let channel = ChannelId(reminder.channel_id);

            channel
                .send_message(&http, |m| m.content(&reminder.content))
                .await;
        }

        async fn send_to_webhook(http: &Http, reminder: &Reminder, webhook: Webhook) {
            webhook
                .execute(&http, false, |w| w.content(&reminder.content))
                .await;
        }

        if let (Some(webhook_id), Some(webhook_token)) = (self.webhook_id, &self.webhook_token) {
            let webhook_res = http.get_webhook_with_token(webhook_id, webhook_token).await;

            if let Ok(webhook) = webhook_res {
                send_to_webhook(http, &self, webhook).await;
            } else {
                warn!("Webhook vanished: {:?}", webhook_res);

                self.reset_webhook(&pool.clone()).await;
                send_to_channel(http, &self).await;
            }
        } else {
            send_to_channel(http, &self).await;
        }

        self.refresh(pool).await;
    }
}
