use serenity::{async_trait, model::prelude::Message};
use tracing::error;

use crate::{
    command::{inner::CmdCtx, Command, FlopMessagable},
    Cli, FlopResult,
};

const ERROR_BYTES: &[u8] = "⚠️**ERROR**⚠️ Unable to serialise value".as_bytes();

#[derive(Debug)]
pub struct MessageCommand<'a> {
    message: FlopMessagable<'a>,
}

#[async_trait]
impl Command for MessageCommand<'_> {
    fn construct(_cli: &Cli, data: &[u8]) -> FlopResult<Self> {
        Ok(Self {
            message: String::from_utf8_lossy(data).into_owned().into(),
        })
    }

    async fn execute<'a>(
        &mut self,
        _msg: &Message,
        _ctx: CmdCtx<'a>,
    ) -> FlopResult<Option<FlopMessagable>> {
        Ok(Some(self.message.clone()))
    }

    fn save(self) -> Vec<u8> {
        match self.message {
            FlopMessagable::Text(s) => s.as_bytes().to_vec(),
            _ => {
                error!(
                    "Not supported value is trying to be seralised: `{:?}`",
                    self.message
                );
                ERROR_BYTES.to_vec()
            }
        }
    }

    // fn raw(&self) -> &str {
    //     match &self.message {
    //         FlopMessagable::Text(s) => s,
    //         _ => {
    //             error!(
    //                 "Not supported value is trying to be seralised: `{:?}`",
    //                 self.message
    //             );
    //             "⚠️**ERROR**⚠️ cannot get raw form of data for command"
    //         }
    //     }
    // }
}
