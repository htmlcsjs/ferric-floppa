use serenity::{all::Mentionable, async_trait, model::channel::Message};
use tokio::sync::RwLock;

use crate::{
    command::{inner::CmdCtx, ExtendedCommand, FlopMessagable},
    sql::FlopDB,
    Cli, FlopResult,
};

#[derive(Debug)]
pub struct RemoveCommand;

#[async_trait]
impl ExtendedCommand for RemoveCommand {
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

        // Special case this command to not cause a mutex gridlock
        if ctx.registry == registry && ctx.name == name.to_lowercase() {
            return Ok(FlopMessagable::Text(
                "Insufficent perms to remove this command".to_owned(),
            ));
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

        // Check cmd owner
        let cmd_lock = cmd.lock().await;
        if cmd_lock.get_owner() != &msg.author.id {
            return Ok(FlopMessagable::Text(format!(
                "⚠️ Cannot remove command, `{name}` is owned by {}",
                cmd.lock().await.get_owner().mention()
            )));
        }
        // Drop lock on cmd to be able to delete it
        drop(cmd_lock);

        // Perform the command deletion
        let mut db_lock = db.write().await;
        if db_lock
            .remove_command(registry.to_owned(), name.to_owned())
            .await
        {
            Ok(FlopMessagable::Text(format!("Deleted command `{name}`")))
        } else {
            Ok(FlopMessagable::Text(format!(
                "Failed to delete command `{name}`"
            )))
        }
    }

    fn save(&self) -> Option<Vec<u8>> {
        None
    }
}
