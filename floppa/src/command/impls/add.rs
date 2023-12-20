use serenity::{all::Mentionable, async_trait, model::channel::Message};
use tokio::sync::RwLock;

use crate::{
    command::{inner::CmdCtx, ExtendedCommand, FlopMessagable},
    sql::FlopDB,
    Cli, FlopResult,
};

use super::MessageCommand;

#[derive(Debug)]
pub struct AddCommand {
    cli: Cli,
}

#[async_trait]
impl ExtendedCommand for AddCommand {
    fn construct(cli: &Cli, _data: &[u8]) -> FlopResult<Self> {
        Ok(Self { cli: cli.clone() })
    }

    async fn execute<'b>(
        &mut self,
        msg: &Message,
        ctx: CmdCtx<'b>,
        db: &RwLock<FlopDB>,
    ) -> FlopResult<Option<FlopMessagable>> {
        let args = msg.content.trim_start_matches(ctx.command).trim();
        let Some((name, body)) = args.split_once(char::is_whitespace) else {
            return Ok(Some(FlopMessagable::Text(format!(
                "Usage: `{} [name] [body]`",
                ctx.command
            ))));
        };

        let mut lock = db.write().await;
        if let Some(cmd) = lock.get_command(name.to_string(), ctx.registry.clone()) {
            return Ok(Some(FlopMessagable::Text(format!(
                "⚠️ `{name}` is already a command, owned by {}",
                cmd.lock().await.get_owner().mention()
            ))));
        }

        let cmd = match MessageCommand::construct(&self.cli, body.as_bytes()) {
            Ok(cmd) => cmd,
            Err(e) => {
                return Ok(Some(FlopMessagable::Text(format!(
                    "⚠️ Error adding command: `{e:?}`"
                ))));
            }
        };

        lock.add_command(
            ctx.registry.to_owned(),
            name.to_string(),
            &msg.author,
            "TextCommand".to_owned(),
            cmd,
        );

        drop(lock);

        Ok(Some(FlopMessagable::Text(format!(
            "✅Added command `{name}`"
        ))))
    }

    fn save(self) -> Vec<u8> {
        Vec::new()
    }
}
