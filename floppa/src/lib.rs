mod command;
pub mod config;

use std::{
    path::{Path, PathBuf},
    process,
    sync::Arc,
};

use async_trait::async_trait;
use clap::Parser;
use serenity::{prelude::*, model::prelude::*};
pub use color_eyre::Result as FlopResult;
pub use config::Config;
use tokio::sync::RwLock;

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
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("connected as {}", ready.user.name);
    }
}