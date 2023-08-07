use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::{Cli, FlopResult, HttpClient, ThreadCfg};

#[async_trait]
pub trait Command<'a> {
    /// This is a type that will hold
    type Data: Serialize + Deserialize<'a>;

    /// Constructs the command from CLI options and config, and any data serialised to disk
    fn construct(cfg: &ThreadCfg, cli: &Cli, data: Self::Data) -> Self;

    /// Allows the command to update itself on config change
    fn cfg_update(&mut self, _cfg: &ThreadCfg) {}

    /// Executes the command on the given MessageCreate event
    async fn execute(&mut self, event: &MessageCreate, http: Arc<HttpClient>) -> FlopResult<()>;

    /// Allows the command to serialise data to be asked
    /// Consumes the command, so it will be reinitalised
    fn save(self) -> Self::Data;

    /// Gets the raw form of the Command
    /// TODO: epic macro to sealise src code at compile time
    fn raw(&self) -> &str;
}

#[derive(Debug)]
struct MessageCommand {
    message: String,
}

#[async_trait]
impl Command<'_> for MessageCommand {
    type Data = String;

    fn construct(_cfg: &ThreadCfg, _cli: &Cli, data: Self::Data) -> Self {
        Self { message: data }
    }

    async fn execute(&mut self, event: &MessageCreate, http: Arc<HttpClient>) -> FlopResult<()> {
        http.create_message(event.channel_id)
            .reply(event.id)
            .content(&self.message)?
            .await?;
        Ok(())
    }

    fn save(self) -> Self::Data {
        self.message
    }

    fn raw(&self) -> &str {
        &self.message
    }
}