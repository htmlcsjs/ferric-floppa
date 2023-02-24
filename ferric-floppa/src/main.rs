mod args;
mod cfg;
mod command;
mod consts;
mod handler;
mod util;

use crate::consts::*;
use args::FlopArgs;
use cfg::FlopConfig;
use clap::Parser;
use command::{SleepCmd, TextCmd};
use handler::Handler;
use serenity::Client;
use tokio::fs;
use tracing::Level;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> FlopResult<()> {
    let args = FlopArgs::parse();
    let cfg: FlopConfig = toml::from_str(&fs::read_to_string(args.run.join("config.toml")).await?)?;
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

    let fmt = tracing_subscriber::fmt().with_env_filter(env_filter);
    #[cfg(debug_assertions)]
    fmt.pretty().try_init()?;
    #[cfg(not(debug_assertions))]
    fmt.try_init()?;

    let mut handler = Handler::init(cfg);

    handler
        .add_cmd(
            "halp".to_owned(),
            TextCmd::new(vec!["Bot is kil".to_owned()]),
        )
        .await;
    handler.add_cmd("sleep".to_owned(), SleepCmd).await;

    let mut client = Client::builder(token, get_intents())
        .event_handler(handler)
        .await?;

    client.start().await?;

    Ok(())
}
