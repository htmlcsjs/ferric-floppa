use serde::{Deserialize, Serialize};

use crate::Cli;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub prefix: String,
    pub logging: LoggingConfig,
}

impl Config {
    pub fn load_from_fs(cli: &Cli) -> anyhow::Result<Self> {
        Ok(serde_yaml::from_reader(std::fs::File::open(
            cli.run_dir.join("config.yaml"),
        )?)?)
    }

    pub async fn write_to_fs(&self, cli: &Cli) -> anyhow::Result<()> {
        tokio::fs::write(
            cli.run_dir.join("config.yaml"),
            serde_yaml::to_string(&self)?,
        )
        .await?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub webhook_url: String,
    pub global_level: String,
    pub webhook_level: String,
}
