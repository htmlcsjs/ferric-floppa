use std::fmt::Debug;

use serenity::{async_trait, model::prelude::*, prelude::*};

use crate::{config::Config, Cli, FlopResult};

#[async_trait]
pub trait Command: Debug {
    /// Constructs the command from CLI options and config, and any data serialised to disk
    fn construct(cfg: &Config, cli: &Cli, data: rmpv::Value) -> Self
    where
        Self: Sized;

    /// Allows the command to update itself on config change
    fn cfg_update(&mut self, _cfg: &Config) {}

    /// Executes the command on the given Message event
    async fn execute(&mut self, event: &Message, ctx: &Context) -> FlopResult<()>;

    /// Allows the command to serialise data to be asked
    /// Consumes the command, so it will be reinitalised
    fn save(self) -> rmpv::Value;

    /// Gets the raw form of the Command
    /// TODO: epic macro to sealise src code at compile time
    fn raw(&self) -> &str;
}

#[derive(Debug)]
pub struct MessageCommand {
    message: String,
}

#[async_trait]
impl Command for MessageCommand {
    fn construct(_cfg: &Config, _cli: &Cli, data: rmpv::Value) -> Self {
        Self {
            message: rmpv::ext::from_value(data).unwrap_or_else(|e| {
                tracing::error!("cannot unpack data for command, {e}");
                format!("⚠️**ERROR**⚠️ cannot unpack data for command ```log\n{e}```")
            }),
        }
    }

    async fn execute(&mut self, msg: &Message, ctx: &Context) -> FlopResult<()> {
        msg.reply(&ctx.http, &self.message).await?;
        Ok(())
    }

    fn save(self) -> rmpv::Value {
        self.message.into()
    }

    fn raw(&self) -> &str {
        &self.message
    }
}
