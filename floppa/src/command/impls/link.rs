use serenity::{all::Mentionable, async_trait, model::channel::Message};
use tokio::sync::RwLock;

use crate::{
    command::{check_name, inner::CmdCtx, ExtendedCommand, FlopMessagable},
    sql::{CmdNode, FlopDB, FlopRole},
    Cli, FlopResult,
};

#[derive(Debug)]
pub struct LinkCommand;

#[async_trait]
impl ExtendedCommand for LinkCommand {
    fn construct(_cli: &Cli, _data: &[u8]) -> FlopResult<Self> {
        Ok(Self)
    }

    // TODO: roles
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
        let Some(name) = args.next() else {
            return Ok(FlopMessagable::Text(format!(
                "Usage: `{0} (name) [registry:](destination`",
                ctx.command
            )));
        };
        let Some(dest) = args.next() else {
            return Ok(FlopMessagable::Text(format!(
                "Usage: `{0} (name) [registry:](destination`",
                ctx.command
            )));
        };

        // check invalid names
        if !check_name(name) {
            return Ok(FlopMessagable::Text(
                "Command names must consist of alphanumeric characters or `-`, `_`".to_string(),
            ));
        }

        // get db log
        let mut lock = db.write().await;
        if !lock.user_has_role(msg.author.id, &FlopRole::RegAdd(ctx.registry.to_owned())) {
            return Ok(FlopMessagable::Text(":clueless:".to_string()));
        }
        if let Some(cmd) = lock.get_command(ctx.registry.to_string(), name.to_string()) {
            return Ok(FlopMessagable::Text(format!(
                "⚠️ `{name}` is already a command, owned by {}",
                cmd.lock().await.get_owner().mention()
            )));
        }

        if dest.is_empty() {
            return Ok(FlopMessagable::Text(
                "No destination command given".to_string(),
            ));
        }

        let (dest_reg, dest_name) = match dest.split_once(':') {
            Some(e) => e,
            None => (ctx.registry, dest),
        };

        if !lock.command_exists(dest_reg.to_owned(), dest_name) {
            return Ok(FlopMessagable::Text(format!(
                "`⚠️ {dest_reg}:{dest_name}` doesnt exist"
            )));
        }

        let node = CmdNode::Symlink {
            reg: dest_reg.to_owned(),
            name: dest_name.to_owned(),
        };

        lock.add_command(
            ctx.registry.to_owned(),
            name.to_string(),
            &msg.author,
            CmdNode::SYMLINK_ID.to_owned(),
            node,
        );

        drop(lock);

        Ok(FlopMessagable::Text(format!(
            "Added link to `{dest_name}` called `{name}`"
        )))
    }

    fn save(&self) -> Option<Vec<u8>> {
        None
    }
}
