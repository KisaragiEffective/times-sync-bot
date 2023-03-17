use serenity::cache::Cache;
use serenity::model::channel::Message;

pub trait AsPermalink {
    type Permalink;
    type Err;

    fn as_permalink(&self, cache: impl AsRef<Cache>) -> Result<Self::Permalink, Self::Err>;
}

impl AsPermalink for Message {
    type Permalink = String;
    type Err = ();

    fn as_permalink(&self, cache: impl AsRef<Cache>) -> Result<Self::Permalink, Self::Err> {
        if let Some(guild) = self.guild(cache) {
            Ok(format!(
                "https://discord.com/channels/{guild_id}/{channel_id}/{message_id}",
                guild_id = guild.id,
                channel_id = self.channel_id,
                message_id = self.id,
            ))
        } else {
            Err(())
        }
    }
}
