mod args;
mod cfg;
mod consts;

use crate::consts::*;
use args::FlopArgs;
use cfg::Config;
use clap::Parser;
use itertools::Itertools;
use serenity::{
    async_trait,
    model::prelude::{Message, ReactionType, Ready},
    prelude::{Context, EventHandler},
    Client,
};
use tokio::fs;
use tracing::{error, info, instrument, Level};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> FlopResult<()> {
    let args = FlopArgs::parse();
    let cfg: Config = toml::from_str(&fs::read_to_string(args.run.join("config.toml")).await?)?;
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
        .event_handler(Handler::init(cfg))
        .await?;

    client.start().await?;

    Ok(())
}

#[derive(Debug)]
struct Handler {
    cfg: Config,
    cache_emoji: ReactionType,
}

impl Handler {
    fn init(cfg: Config) -> Self {
        let cache_emoji = ReactionType::Custom {
            animated: cfg.emote.animated,
            id: cfg.emote.id.into(),
            name: Some(cfg.emote.name.clone()),
        };

        Self { cfg, cache_emoji }
    }
}

#[async_trait]
impl EventHandler for Handler {
    #[instrument(skip_all)]
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!amogus" && !msg.author.bot {
            if let Err(e) = msg.reply(&ctx.http, "imposterious").await {
                error!("Error sending message: {e:?}");
            }
        }
        if msg
            .content
            .chars()
            .dedup()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
            .contains(&self.cfg.emote.phrase)
        {
            if let Err(e) = msg.react(&ctx.http, self.cache_emoji.clone()).await {
                error!("Error adding reaction: {e:?}");
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
