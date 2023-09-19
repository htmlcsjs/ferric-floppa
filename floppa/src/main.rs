mod command;
pub mod config;
mod log;

use std::{
    path::{Path, PathBuf},
    process,
    sync::Arc,
};

use clap::Parser;
pub use color_eyre::Result as FlopResult;
use config::Config;
use log::FlopLog;
use serenity::{async_trait, model::prelude::*, prelude::*};
use tokio::{fs, sync::RwLock};
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    color_eyre::install().expect("failed to install panic handler");

    let cli = match Cli::initlise() {
        Ok(inner) => inner,
        Err(e) => panic!("Fatal error loading cli args:\n{e}"),
    };

    let cfg = match Config::load_from_fs(&cli) {
        Ok(inner) => inner,
        Err(e) => panic!("Fatal error during initial config loading:\n{e}"),
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(FlopLog::new(&cfg))
        .init();

    if let Err(e) = run(cli, cfg).await {
        tracing::error!("Fatal error encountered: {e}")
    }
}

async fn run(cli: Cli, cfg: Config) -> FlopResult<()> {
    // TODO: Have a default for this
    let _cfg = Arc::new(RwLock::new(cfg));

    let temp_token = fs::read_to_string(cli.get_path("token")).await?;
    let token = temp_token.trim().to_string();

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(FlopHandler)
        .await?;

    client.start().await?;

    Ok(())
}

pub type ThreadCfg = Arc<RwLock<Config>>;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(
        short,
        long,
        value_name = "PATH",
        default_value = ".",
        hide_default_value = true
    )]
    /// Sets the directory to be used as the base at runtime.
    /// Default is the current working directory
    run_dir: PathBuf,
}

impl Cli {
    pub fn initlise() -> FlopResult<Self> {
        let mut new = Self::parse();
        new.run_dir = new.run_dir.canonicalize()?;
        if !new.run_dir.is_dir() {
            tracing::error!("{} is not a directory!", new.run_dir.display());
            process::exit(1);
        }
        Ok(new)
    }

    #[inline]
    pub fn get_path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.run_dir.join(path)
    }
}

pub struct FlopHandler;

#[async_trait]
impl EventHandler for FlopHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!ping" {
            if let Err(why) = msg.channel_id.say(&ctx.http, "Pong!").await {
                tracing::error!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        tracing::info!("Connected as {}", ready.user.name);
    }
}