use chrono::Local;
use log::{debug, error, info, warn};
use serenity::async_trait;
use serenity::client::{Context, EventHandler};
use serenity::constants::MESSAGE_CODE_LIMIT;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use crate::Args;
use crate::permalink::AsPermalink;


pub struct Syncer(pub Args);

#[async_trait]
impl EventHandler for Syncer {
    async fn message(&self, ctx: Context, incoming_message: Message) {
        let author_user_id = incoming_message.author.id;
        let author_bot = incoming_message.author.bot;
        if author_bot {
            if !self.0.explicit_opt_in_bot.contains(&author_user_id) {
                debug!("author ({author_user_id}) is not target BOT, skipping");
                return
            }
        } else {
            if !self.0.include_user.contains(&author_user_id) {
                debug!("author ({author_user_id}) is not target, skipping");
                return
            }
        }

        let channel_id = incoming_message.channel_id;
        if channel_id != self.0.original_channel {
            debug!("posted in non-target channel ({channel_id}), skipping");
            return
        }

        info!("sync started. permalink: {permalink}", permalink = incoming_message.as_permalink(&ctx).unwrap());
        let iso_date = Local::now().to_rfc3339();
        let original_content = incoming_message.content;
        let mut new_message = format!(r#"----------
Subject: このメッセージは <#{channel_id}> から転送されたメッセージです。
Date: {iso_date}
Age: 0
Content-Type: text/markdown

----------
{original_content}
"#);
        const TOO_LONG: &str = "長すぎるため省略されました。";
        let new_message = if new_message.len() > MESSAGE_CODE_LIMIT {
            let maximum_unicode_codepoint_length = MESSAGE_CODE_LIMIT - TOO_LONG.chars().count() - 1;
            let new_message = new_message.chars().take(maximum_unicode_codepoint_length).collect::<String>();
            format!("{new_message}\n{TOO_LONG}")
        } else {
            new_message
        };

        for channel_id in &self.0.sync_destination_channel {
            info!(
                "trying {channel_id} (#{channel_name})",
                channel_name = channel_id.name(&ctx).await.unwrap_or("<<unknown channel name>>".to_string()),
            );

            let deliver_result = channel_id.send_message(&ctx, |b|
                b
                    .content(new_message.as_str())
                    .allowed_mentions(|m|
                        // do not interpret mention
                        m.empty_parse()
                    )
            ).await;

            match deliver_result {
                Ok(mut m) => {
                    let g = ctx.cache.message(m.channel_id, m.id).and_then(|m| m.guild_id);
                    m.guild_id = g;
                    if let Ok(permalink) = m.as_permalink(&ctx) {
                        info!("success: delivered as {permalink}");
                    } else {
                        info!("success: delivered (could not build permalink)");
                    }
                }
                Err(e) => {
                    error!("failed to sync: {e:?}");
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, data_about_bot: Ready) {
        info!("connected");
        info!("bot.id: {}", data_about_bot.user.id);
        let guilds = &data_about_bot.guilds;
        let joined_guild_counts = guilds.len();
        if joined_guild_counts == 0 {
            warn!("this bot does not belong to ANY guild!");
            return
        }
        info!("connected guilds ({})", joined_guild_counts);
        for guild in guilds {
            info!("- id: {id} ({name})", id = guild.id, name = guild.id.name(&ctx).unwrap_or("<<unavailable guild name>>".to_string()));
        }
    }
}
