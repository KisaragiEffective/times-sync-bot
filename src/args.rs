use clap::{ArgMatches, Error, Parser};
use serenity::model::id::{ChannelId, UserId};

#[derive(Parser)]
pub struct Args {
    #[clap(long = "from")]
    pub original_channel: ChannelId,
    #[clap(long, long = "to")]
    pub sync_destination_channel: Vec<ChannelId>,
    #[clap(long, long = "bot")]
    pub explicit_opt_in_bot: Vec<UserId>,
    #[clap(long, long = "user")]
    pub include_user: Vec<UserId>,
}
