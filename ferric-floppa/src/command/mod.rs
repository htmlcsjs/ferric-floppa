use std::fmt::Debug;

use serenity::{async_trait, model::prelude::Message, prelude::Context};

mod text_cmds;

pub use text_cmds::*;

#[async_trait]
pub trait FlopCommand: Debug + Sync + Send {
    // TODO support editing
    async fn execute(&self, msg: Message, ctx: &Context);
}
