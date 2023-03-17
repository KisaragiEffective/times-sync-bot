mod event;
mod permalink;
mod args;

use std::error::Error;
use chrono::Local;
use clap::Parser;
use fern::colors::ColoredLevelConfig;
use log::Level;
use serenity::client::{ClientBuilder, EventHandler};
use serenity::prelude::GatewayIntents;
use crate::args::Args;
use crate::event::Syncer;

fn initialize_fern() -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::new();

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                colors.color(record.level()),
                message,
            ))
        })
        .level(log::LevelFilter::Info)
        // skip log record from noisy serenity shard runner
        .filter(|x| x.target() != "tracing::span" && x.level() == Level::Info)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync + 'static>> {
    initialize_fern()?;

    let args = Args::parse();
    let discord_token = std::env::var("DISCORD_TOKEN").expect("$DISCORD_TOKEN is required");
    // in order to receive message, specify GUILD_MESSAGES
    // in order to receive guild_id, specify GUILDS
    let mut client = ClientBuilder::new(discord_token, GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILDS)
        .event_handler(Syncer(args))
        .await?;

    client.start().await?;

    Ok(())
}
