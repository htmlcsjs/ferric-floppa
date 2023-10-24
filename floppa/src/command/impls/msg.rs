use serenity::{async_trait, model::prelude::Message, prelude::*};
use tracing::error;

use crate::{
    command::{Command, FlopMessagable},
    config::Config,
    Cli, FlopResult,
};

#[derive(Debug)]
pub struct MessageCommand<'a> {
    message: FlopMessagable<'a>,
}

#[async_trait]
impl Command for MessageCommand<'_> {
    fn construct(_cfg: &Config, _cli: &Cli, data: rmpv::Value) -> Self {
        Self {
            message: rmpv::ext::from_value(data)
                .unwrap_or_else(|e| {
                    tracing::error!("cannot unpack data for command, {e}");
                    format!("⚠️**ERROR**⚠️ cannot unpack data for command ```log\n{e}```")
                })
                .into(),
        }
    }

    async fn execute(
        &mut self,
        _msg: &Message,
        _ctx: &Context,
    ) -> FlopResult<Option<FlopMessagable>> {
        Ok(Some(self.message.clone()))
    }

    fn save(self) -> rmpv::Value {
        match self.message {
            FlopMessagable::Text(s) => s.into(),
            _ => {
                error!(
                    "Not supported value is trying to be seralised: `{:?}`",
                    self.message
                );
                rmpv::Value::Nil
            }
        }
    }

    fn raw(&self) -> &str {
        match &self.message {
            FlopMessagable::Text(s) => s,
            _ => {
                error!(
                    "Not supported value is trying to be seralised: `{:?}`",
                    self.message
                );
                "⚠️**ERROR**⚠️ cannot get raw form of data for command"
            }
        }
    }
}
