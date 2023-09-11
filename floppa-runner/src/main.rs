mod log;

use std::sync::Arc;

use floppa::{Cli, Config, FlopResult, FlopHandler};
use log::FlopLog;
use tokio::{fs, sync::RwLock};
use tracing_subscriber::prelude::*;
use serenity::prelude::*;

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

    let mut client = Client::builder(&token, intents).event_handler(FlopHandler).await?;

    client.start().await?;

    Ok(())
}
