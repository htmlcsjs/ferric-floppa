use serenity::{async_trait, model::prelude::Message};
use tracing::{error, warn};

use crate::{
    command::{inner::CmdCtx, Command, FlopMessagable},
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

    async fn execute<'a>(
        &mut self,
        _msg: &Message,
        _ctx: CmdCtx<'a>,
    ) -> FlopResult<FlopMessagable> {
        warn!("Somone has managed to run a marker command");
        Ok(FlopMessagable::None)
    }

    fn save(&self) -> Option<Vec<u8>> {
        match rmp_serde::to_vec_named(&self) {
            Ok(data) => Some(data),
            Err(e) => {
                error!("Failed to serialise symlink data {e}");
                None
            }
        }
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

    async fn execute<'a>(
        &mut self,
        _msg: &Message,
        _ctx: CmdCtx<'a>,
    ) -> FlopResult<FlopMessagable> {
        warn!("Someone has managed to run a marker command");
        Ok(FlopMessagable::None)
    }

    fn save(&self) -> Option<Vec<u8>> {
        Some(self.registry.as_bytes().to_vec())
    }
}
