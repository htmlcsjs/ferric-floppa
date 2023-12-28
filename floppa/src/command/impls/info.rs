use serenity::{all::Mentionable, async_trait, model::channel::Message};
use tokio::sync::RwLock;

use crate::{
    command::{inner::CmdCtx, ExtendedCommand, FlopMessagable},
    sql::FlopDB,
    Cli, FlopResult,
};

#[derive(Debug)]
pub struct InfoCommand;

#[async_trait]
impl ExtendedCommand for InfoCommand {
    fn construct(_cli: &Cli, _data: &[u8]) -> FlopResult<Self> {
        Ok(Self)
    }

    async fn execute<'b>(
        &mut self,
        msg: &Message,
        ctx: CmdCtx<'b>,
        db: &RwLock<FlopDB>,
    ) -> FlopResult<FlopMessagable> {
        let mut args = msg
            .content
            .trim_start_matches(ctx.command)
            .split_whitespace();

        let Some(mut name) = args.next() else {
            return Ok(FlopMessagable::Text(format!(
                "Usage {} [registry:](command)",
                ctx.command
            )));
        };

        let mut registry = ctx.registry;
        if let Some((new_reg, new_name)) = name.split_once(':') {
            registry = new_reg;
            name = new_name;
        }

        let db_lock = db.read().await;
        let Some(cmd) = db_lock.get_command(registry.to_owned(), name.to_owned()) else {
            let end = if registry != ctx.registry {
                format!(" in registry `{registry}`")
            } else {
                "".to_owned()
            };

            return Ok(FlopMessagable::Text(format!(
                "⚠️ Failed to find command `{name}`{end}"
            )));
        };

        // Drop lock to free db for other uses
        drop(db_lock);

        // Special case this command to not cause a mutex gridlock
        if ctx.registry == registry && ctx.name == name.to_lowercase() {
            return Ok(FlopMessagable::Text(format!(
                "Command `{}` was added at <t:{}:f>, and is owned by {}",
                ctx.name,
                ctx.added,
                ctx.owner.mention()
            )));
        }

        let cmd_lock = cmd.lock().await;

        let owner = cmd_lock.get_owner();
        let registry = cmd_lock.get_registry();
        let name = cmd_lock.get_name();
        let added = cmd_lock.get_added();

        let mut msg = format!("Command `{name}`");
        if registry != ctx.registry {
            msg += &format!(" in registry `{registry}`")
        }

        msg += &format!(
            " was added at <t:{added}:f>, and is owned by {}",
            owner.mention()
        );

        Ok(FlopMessagable::Text(msg))
    }

    fn save(&self) -> Option<Vec<u8>> {
        None
    }
}
