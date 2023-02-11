mod args;
mod consts;

use crate::consts::*;
use args::FlopArgs;
use clap::Parser;
use serenity::{
    async_trait,
    model::prelude::{Message, Ready},
    prelude::{Context, EventHandler},
    Client,
};
use tokio::fs;
use tracing::{error, info, instrument, Level};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> FlopResult<()> {
    let args = FlopArgs::parse();
    let mut token = fs::read_to_string(args.run.join("token")).await?;
    token.retain(|c| !c.is_whitespace());

    let other_crates_level = match args.log_level {
        Level::ERROR => Level::ERROR,
        Level::WARN => Level::WARN,
        Level::INFO => Level::WARN,
        Level::DEBUG => Level::INFO,
        Level::TRACE => Level::TRACE,
    };

    let env_filter = EnvFilter::builder()
        .with_default_directive(other_crates_level.into())
        .parse(
            other_crates_level.to_string()
                + concat!(",", env!("CARGO_CRATE_NAME"), "=")
                + &args.log_level.to_string(),
        )?;

    tracing_subscriber::fmt()
        .pretty()
        .with_env_filter(env_filter)
        .try_init()?;

    let mut client = Client::builder(token, get_intents())
        .event_handler(Handler)
        .await?;

    client.start().await?;

    Ok(())
}

#[derive(Debug)]
struct Handler;

#[async_trait]
impl EventHandler for Handler {
    #[instrument(skip_all)]
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!amogus" {
            if let Err(why) = msg.reply(&ctx.http, "imposterious").await {
                error!("Error sending message: {why:?}");
            }
        }
    }

    #[instrument(skip_all)]
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!(
            "Logged in as {}#{:04}",
            ready.user.name, ready.user.discriminator
        );
    }
}
