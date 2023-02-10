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
use tracing::{error, info, instrument};

#[tokio::main]
async fn main() -> FlopResult<()> {
    let args = FlopArgs::parse();
    let mut token = fs::read_to_string(args.run.join("token")).await?;
    token.retain(|c| !c.is_whitespace());

    tracing_subscriber::fmt()
        .with_max_level(args.log_level)
        .pretty()
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
        info!("{} is connected!", ready.user.name);
    }
}
