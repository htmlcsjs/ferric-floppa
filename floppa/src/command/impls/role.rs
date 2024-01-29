use serenity::{all::Mentionable, async_trait, model::channel::Message};
use tokio::sync::RwLock;

use crate::{
    command::{inner::CmdCtx, ExtendedCommand, FlopMessagable},
    sql::{FlopDB, FlopRole},
    stuff, Cli, FlopResult,
};

#[derive(Debug)]
pub struct RoleCommand;

#[async_trait]
impl ExtendedCommand for RoleCommand {
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
        let Some(user) = args.next() else {
            return Ok(FlopMessagable::Text(format!(
                "Usage: `{0} (user) (role)` or `{0} (user) -r (role)` to remove role",
                ctx.command
            )));
        };
        let Some(mut role) = args.next() else {
            return Ok(FlopMessagable::Text(format!(
                "Usage: `{0} (user) (role)` or `{0} (user) -r (role)` to remove role",
                ctx.command
            )));
        };

        let mut rm_mode = role == "-r";
        if rm_mode {
            role = match args.next() {
                None => {
                    return Ok(FlopMessagable::Text(format!(
                        "Usage: `{0} (user) (role)` or `{0} (user) -r (role)` to remove role",
                        ctx.command
                    )))
                }
                Some(r) => r,
            };
        } else if args.next().map(|x| x == "-r").unwrap_or_default() {
            rm_mode = true;
        }

        let Some(user) = stuff::try_get_user(user) else {
            return Ok(FlopMessagable::Text(
                "Could not find user, only mentions or id is valid".to_string(),
            ));
        };

        let Some(role) = FlopRole::from_str(role) else {
            return Ok(FlopMessagable::Text(format!(
                "Didn't understand role `{role}`"
            )));
        };

        // get db lock
        let mut db_lock = db.write().await;

        if !db_lock.user_has_role(msg.author.id, &FlopRole::Admin) {
            return Ok(FlopMessagable::Text(":clueless:".to_string()));
        }
        if !rm_mode {
            db_lock.give_role(user, role);
        } else {
            db_lock.remove_role(user, role)
        }

        drop(db_lock);

        Ok(FlopMessagable::Text(format!(
            "Gave role to `{}`",
            user.mention()
        )))
    }

    fn save(&self) -> Option<Vec<u8>> {
        None
    }
}
