mod command;
pub mod config;

use std::{
    error::Error,
    path::{Path, PathBuf},
    process,
    sync::Arc,
};

use clap::Parser;
pub use color_eyre::Result as FlopResult;
pub use config::Config;
use tokio::sync::RwLock;
use twilight_gateway::Event;
pub use twilight_http::Client as HttpClient;

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
pub async fn handle_event(
    event: Event,
    http: Arc<HttpClient>,
    cfg: ThreadCfg,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let prefix = {
        let handle = cfg.read().await;
        handle.prefix.to_string()
    };

    match event {
        Event::MessageCreate(msg) if msg.content.starts_with(&prefix) => {
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
