use serde::{Deserialize, Serialize};

use crate::{Cli, FlopResult};

#[derive(Debug, Serialize, Deserialize)]
/// Global config for floppa
pub struct Config {
    /// The prefix for commands.
    pub prefix: String,
    /// The amount of messages to cache per channel
    pub msg_cache: usize,
    /// See [`LoggingConfig`]
    pub logging: LoggingConfig,
    /// See [`EmojiConfig`]
    pub emoji: EmojiConfig,
}

impl Config {
    pub fn load_from_fs(cli: &Cli) -> FlopResult<Self> {
        Ok(serde_yaml::from_reader(std::fs::File::open(
            cli.run_dir.join("config.yaml"),
        )?)?)
    }

    pub async fn write_to_fs(&self, cli: &Cli) -> FlopResult<()> {
        tokio::fs::write(
            cli.run_dir.join("config.yaml"),
            serde_yaml::to_string(&self)?,
        )
        .await?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
/// Config for printing [`tracing`] logs to a webhook
pub struct LoggingConfig {
    /// The url of the webhook
    pub webhook_url: String,
    /// The min level to be printed to stdout
    pub global_level: String,
    /// The min level to be sent to the webhook
    pub webhook_level: String,
}

#[derive(Debug, Serialize, Deserialize)]
/// config for reacting to messages that contain a phrase
pub struct EmojiConfig {
    /// The textual representation of the emoji to react with
    pub emoji: String,
    /// What activates the reaction
    pub phrase: String,
}
