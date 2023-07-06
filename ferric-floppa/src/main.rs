#![feature(trait_alias)]
mod config;

use std::{
    error::Error,
    path::{Path, PathBuf},
    process,
    sync::Arc,
};

use clap::Parser;
use config::Config;
use tokio::{fs, sync::RwLock};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Event, Shard, ShardId};
use twilight_http::Client as HttpClient;
use twilight_model::gateway::Intents;

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
    fn initlise() -> anyhow::Result<Self> {
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    if let Err(e) = run().await {
        tracing::error!("Fatal error encountered: {e}")
    }
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::initlise()?;
    // TODO: Have a default for this
    let cfg = Arc::new(RwLock::new(Config::load_from_fs(&cli)?));

    let temp_token = fs::read_to_string(cli.get_path("token")).await?;
    let token = temp_token.trim().to_string();

    let mut shard = Shard::new(
        ShardId::ONE,
        token.clone(),
        Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT,
    );

    let http = Arc::new(HttpClient::new(token));

    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();

    loop {
        let event = match shard.next_event().await {
            Ok(event) => event,
            Err(source) => {
                tracing::warn!(?source, "error receiving event");

                if source.is_fatal() {
                    break;
                }

                continue;
            }
        };

        cache.update(&event);

        tokio::spawn(handle_event(event, Arc::clone(&http), Arc::clone(&cfg)));
    }

    Ok(())
}

async fn handle_event(
    event: Event,
    http: Arc<HttpClient>,
    cfg: Arc<RwLock<Config>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let prefix = {
        let handle = cfg.read().await;
        handle.prefix.clone()
    };

    match event {
        Event::MessageCreate(msg) if msg.content == prefix + "ping" => {
            http.create_message(msg.channel_id)
                .reply(msg.id)
                .content(":flop:")?
                .await?;
        }
        // Other events here...
        Event::Ready(ready) => {
            tracing::info!("Logged in as {}", ready.user.name);
        }
        _ => (),
    }

    Ok(())
}
