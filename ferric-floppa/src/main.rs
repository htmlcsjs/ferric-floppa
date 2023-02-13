mod args;
mod cfg;
mod consts;
mod handler;

use crate::consts::*;
use args::FlopArgs;
use cfg::Config;
use clap::Parser;
use itertools::Itertools;
use serenity::Client;
use tokio::fs;
use tracing::Level;
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
