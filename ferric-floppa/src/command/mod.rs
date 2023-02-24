use std::fmt::Debug;

use serenity::{async_trait, model::prelude::Message, prelude::Context};

use crate::cfg::FlopConfig;
mod text_cmds;

pub use text_cmds::*;

#[async_trait]
pub trait DataHolder: Sync + Send {
    async fn get_cfg(&self) -> FlopConfig;
    async fn set_cfg(&mut self, new_cfg: FlopConfig);
}

#[async_trait]
pub trait FlopCommand: Debug + Sync + Send {
    // TODO support editing
    async fn execute(&self, data: &dyn DataHolder, msg: Message, ctx: &Context);
}

// TODO make this a proc macro for that sweet syntax
#[macro_export]
macro_rules! handle_error {
    ($src:expr) => {
        if let Err(e) = $src {
            tracing::error!("Error during command execution: {e:}");
        }
    };
    ($src:expr, $emsg:literal) => {
        if let Err(e) = $src {
            tracing::error!("{}: {e:}", $emsg);
        }
    };
    ($src:expr, $dmsg:ident) => {
        if let Err(e) = $src {
            tracing::error!("Error during execution of `{}`: {e:}", $dmsg.link());
        }
    };
    ($src:expr, $emsg:literal, $dmsg:ident) => {
        if let Err(e) = $src {
            tracing::error!("{} while executing `{}`: {e:}", $emsg, $dmsg.link());
        }
    };
}
