use std::sync::atomic::Ordering;

use serenity::{async_trait, model::prelude::Message};

use crate::{
    command::{inner::CmdCtx, Command, FlopMessagable},
    handler::REACTION_COUNT,
    Cli, FlopResult,
};

#[derive(Debug)]
pub struct FlopCountCommand;

#[async_trait]
impl Command for FlopCountCommand {
    fn construct(_cli: &Cli, _data: &[u8]) -> FlopResult<Self> {
        Ok(Self)
    }

    async fn execute<'a>(
        &mut self,
        _msg: &Message,
        _ctx: CmdCtx<'a>,
    ) -> FlopResult<FlopMessagable> {
        Ok(FlopMessagable::Text(format!(
            "{} flops reacted to since last reset",
            REACTION_COUNT.load(Ordering::Relaxed)
        )))
    }

    fn save(&self) -> Option<Vec<u8>> {
        None
    }
}
