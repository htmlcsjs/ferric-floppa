use serenity::{async_trait, model::prelude::Message, prelude::*};
use tracing::{error, warn};

use crate::{
    command::{Command, FlopMessagable},
    Cli, FlopResult,
};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
/// Should not be used to actually run as a command, just to indicate that it is
/// a symlink to another
pub struct RedirectMarkerCommand {
    registry: String,
    command: String,
}

#[async_trait]
impl Command for RedirectMarkerCommand {
    fn construct(_cli: &Cli, data: &[u8]) -> FlopResult<Self> {
        Ok(rmp_serde::from_slice(data)?)
    }

    async fn execute(
        &mut self,
        _msg: &Message,
        _ctx: &Context,
    ) -> FlopResult<Option<FlopMessagable>> {
        warn!("Somone has managed to run a marker command");
        Ok(None)
    }

    fn save(self) -> Vec<u8> {
        match rmp_serde::to_vec_named(&self) {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to serialise symlink data {e}");
                vec![]
            }
        }
    }

    fn raw(&self) -> &str {
        "TODO"
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
/// Should not be used to actually run as a command, just to indicate that it is
/// a subregistry
pub struct SubregistyMarkerCommand {
    registry: String,
}

#[async_trait]
impl Command for SubregistyMarkerCommand {
    fn construct(_cli: &Cli, data: &[u8]) -> FlopResult<Self> {
        Ok(Self {
            registry: String::from_utf8(data.to_vec())?,
        })
    }

    async fn execute(
        &mut self,
        _msg: &Message,
        _ctx: &Context,
    ) -> FlopResult<Option<FlopMessagable>> {
        warn!("Somone has managed to run a marker command");
        Ok(None)
    }

    fn save(self) -> Vec<u8> {
        self.registry.into_bytes()
    }

    fn raw(&self) -> &str {
        "TODO"
    }
}
