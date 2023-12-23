use serenity::{all::Mentionable, async_trait, model::channel::Message};
use tokio::sync::RwLock;

use crate::{
    command::{inner::CmdCtx, ExtendedCommand, FlopMessagable},
    sql::FlopDB,
    Cli, FlopResult,
};

use super::MessageCommand;

#[derive(Debug)]
pub struct EditCommand {
    cli: Cli,
}

#[async_trait]
impl ExtendedCommand for EditCommand {
    fn construct(cli: &Cli, _data: &[u8]) -> FlopResult<Self> {
        Ok(Self { cli: cli.clone() })
    }

    // TODO: roles
    async fn execute<'b>(
        &mut self,
        msg: &Message,
        ctx: CmdCtx<'b>,
        db: &RwLock<FlopDB>,
    ) -> FlopResult<FlopMessagable> {
        let args = msg.content.trim_start_matches(ctx.command).trim();
        let Some((name, body)) = args.split_once(char::is_whitespace) else {
            return Ok(FlopMessagable::Text(format!(
                "Usage: `{} (name) (body)`",
                ctx.command
            )));
        };

        // Special case this command to not cause a mutex gridlock
        if ctx.name == name.to_lowercase() {
            return Ok(FlopMessagable::Text(
                "Insufficent perms to edit this command".to_owned(),
            ));
        }

        // get db lock
        let lock = db.write().await;
        let Some(cmd) = lock.get_command(ctx.registry.to_string(), name.to_string()) else {
            return Ok(FlopMessagable::Text(format!("⚠️ `{name}` is not a command")));
        };
        drop(lock);

        // Check if the command is owned by the person executing the command
        let mut cmd_lock = cmd.lock().await;
        if cmd_lock.get_owner() != &msg.author.id {
            return Ok(FlopMessagable::Text(format!(
                "⚠️ `{name}` is owned by {}",
                cmd.lock().await.get_owner().mention()
            )));
        }

        // Check if old command was a text command
        if cmd_lock.get_type() != stringify!(MessageCommand) {
            return Ok(FlopMessagable::Text(format!(
                "⚠️ `{name}` is not a text command"
            )));
        }

        // Construct new command
        let new_cmd = match MessageCommand::construct(&self.cli, body.as_bytes()) {
            Ok(new_cmd) => new_cmd,
            Err(e) => {
                return Ok(FlopMessagable::Text(format!(
                    "⚠️ Error constructing command: `{e:?}`"
                )));
            }
        };

        *cmd_lock.get_inner() = Box::new(new_cmd);
        cmd_lock.mark_dirty();

        // Drop command lock
        drop(cmd_lock);

        Ok(FlopMessagable::Text(format!("Edited command `{name}`")))
    }

    fn save(self) -> Vec<u8> {
        Vec::new()
    }
}
