use std::fmt::Debug;

use serenity::{async_trait, builder::CreateMessage, model::prelude::Message};

use crate::{cfg::FlopConfig, consts::FlopResult};
mod text_cmds;

pub use text_cmds::*;

#[async_trait]
pub trait DataHolder {
    async fn get_cfg(&self) -> FlopConfig;
    async fn set_cfg(&mut self, new_cfg: FlopConfig);
}

pub trait FlopCommand: Debug + Sync + Send {
    // TODO support editing
    fn execute(&self, data: &dyn DataHolder, msg: Message, m: &mut CreateMessage)
        -> FlopResult<()>;
}
