mod command;
pub mod config;
mod handler;
mod log;
mod sql;

use std::{
    path::{Path, PathBuf},
    process,
};

use clap::Parser;
pub use color_eyre::Result as FlopResult;
use config::Config;
use handler::FlopHandler;
use log::FlopLog;
use serenity::{cache::Settings as CacheSettings, model::prelude::*, prelude::*};
use tokio::{fs, join, runtime::Handle};
use tracing::{error, warn};
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    color_eyre::install().expect("failed to install panic handler");

    let cli = match Cli::initlise() {
        Ok(inner) => inner,
        Err(e) => panic!("Fatal error loading cli args:\n{e:?}"),
    };

    let cfg = match Config::load_from_fs(&cli) {
        Ok(inner) => inner,
        Err(e) => panic!("Fatal error during initial config loading:\n{e}"),
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(FlopLog::new(&cfg))
        .init();

    // TODO: Have a default for this
    // let _cfg = Arc::new(RwLock::new(cfg));

    let temp_token = fs::read_to_string(cli.get_path("token"))
        .await
        .expect("Error reading token");
    let token = temp_token.trim().to_string();

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut cache_settings = CacheSettings::default();

    cache_settings.max_messages = cfg.msg_cache;

    let handler = FlopHandler::new(cfg.clone(), cli).await;
    let db = handler.get_db();

    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .cache_settings(cache_settings)
        .await
        .expect("Error building Client");

    // Spawn task to consistantly sync db
    let a = tokio::spawn(handler::db_sync_loop(cfg.save_duration, db.clone()));

    // Spawn the main task
    let moved_db = db.clone();
    let b = tokio::spawn(async move {
        if let Err(e) = client.start().await {
            error!("Fatal error running client```rust\n{e}```")
        }
        handler::db_sync(moved_db).await;
    });

    // Set the ctrl+c handler
    let handle = Handle::current();
    let abort = vec![a.abort_handle(), b.abort_handle()];
    ctrlc::set_handler(move || {
        warn!("terminating floppa");
        handle.block_on(handler::db_sync(db.clone()));
        let _enter = handle.enter();
        for handle in &abort {
            handle.abort();
        }
    })
    .expect("error setting ctrlc handler");

    let joined = join!(a, b);
    if let Err(e) = joined.0.and(joined.1) {
        if !e.is_cancelled() {
            let msg = format!("Error waiting for tasks: {e}");
            error!("{}", msg);
        }
    }
}

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
            error!("{} is not a directory!", new.run_dir.display());
            process::exit(1);
        }
        Ok(new)
    }

    #[inline]
    pub fn get_path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.run_dir.join(path)
    }
}
