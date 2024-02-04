use serenity::{all::Message, async_trait};

use crate::{
    command::{CmdCtx, Command, FlopMessagable},
    Cli, FlopResult,
};

const CRATE_VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_VERSION: &str = env!("GIT_HASH");

#[derive(Debug)]
pub struct VersionCommand;

#[async_trait]
impl Command for VersionCommand {
    fn construct(_cli: &Cli, _data: &[u8]) -> FlopResult<Self> {
        Ok(Self)
    }

    async fn execute<'a>(&mut self, msg: &Message, ctx: CmdCtx<'a>) -> FlopResult<FlopMessagable> {
        let name = {
            if let Ok(current_user) = ctx
                .ctx
                .http
                .get_current_user_guild_member(msg.guild_id.unwrap_or_default())
                .await
            {
                current_user.display_name().to_string()
            } else {
                "Issue Flop".to_string()
            }
        };
        if GIT_VERSION.is_empty() {
            Ok(FlopMessagable::Text(format!(
                "{name} is running on version `{CRATE_VERSION}`",
            )))
        } else {
            Ok(FlopMessagable::Text(format!(
                "{name} is running on version `{CRATE_VERSION}`, commit `{GIT_VERSION}`",
            )))
        }
    }

    fn save(&self) -> Option<Vec<u8>> {
        None
    }
}
