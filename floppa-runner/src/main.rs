mod log;

use std::sync::Arc;

use floppa::{Cli, Config, FlopResult};
use log::FlopLog;
use tokio::{fs, sync::RwLock};
use tracing_subscriber::prelude::*;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Shard, ShardId};
pub use twilight_http::Client as HttpClient;
use twilight_model::gateway::Intents;

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
    let cfg = Arc::new(RwLock::new(cfg));

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

        tokio::spawn(floppa::handle_event(
            event,
            Arc::clone(&http),
            Arc::clone(&cfg),
        ));
    }

    Ok(())
}
