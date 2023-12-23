use serenity::{all::Mentionable, async_trait, model::channel::Message};
use tokio::sync::RwLock;

use crate::{
    command::{check_name, construct, inner::CmdCtx, ExtendedCommand, FlopMessagable, VALID},
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
                "Usage: `{0} (name) (body)`\nor`{0} (name) (--[type]) [json data]",
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
        if let Some(cmd) = lock.get_command(ctx.registry.to_string(), name.to_string()) {
            return Ok(FlopMessagable::Text(format!(
                "⚠️ `{name}` is already a command, owned by {}",
                cmd.lock().await.get_owner().mention()
            )));
        }

        // deal with other command types
        if body.starts_with("--[") {
            let (ty, body) = match body.split_once(char::is_whitespace) {
                Some((ty, body)) => (&ty.trim()[..(ty.len() - 2)][3..], body),
                None => (&body.trim()[..(body.len() - 1)][3..], ""),
            };

            // deal with json data
            let mut data: Vec<u8> = vec![];
            if !body.is_empty() {
                let value = match serde_json::from_str::<rmpv::Value>(body) {
                    Ok(val) => val,
                    Err(e) => {
                        return Ok(FlopMessagable::Text(format!(
                            "⚠️ Error deseralising json data: ```{e}```"
                        )))
                    }
                };

                data = match rmp_serde::to_vec(&value) {
                    Ok(val) => val,
                    Err(e) => {
                        return Ok(FlopMessagable::Text(format!(
                            "⚠️ Error seralising msgpack data: ```{e}```"
                        )))
                    }
                };
            }

            if !VALID.contains(&ty) {
                return Ok(FlopMessagable::Text(format!(
                    "⚠️ `{ty}` is not a valid command type"
                )));
            }

            let cmd = match construct(ty, &data, &self.cli) {
                Ok(cmd) => cmd,
                Err(e) => {
                    return Ok(FlopMessagable::Text(format!(
                        "⚠️ Error adding command: `{e:?}`"
                    )));
                }
            };

            lock.add_command(
                ctx.registry.to_owned(),
                name.to_string(),
                &msg.author,
                ty.to_owned(),
                cmd,
            );
        } else {
            let cmd = match MessageCommand::construct(&self.cli, body.as_bytes()) {
                Ok(cmd) => cmd,
                Err(e) => {
                    return Ok(FlopMessagable::Text(format!(
                        "⚠️ Error adding command: `{e:?}`"
                    )));
                }
            };

            lock.add_command(
                ctx.registry.to_owned(),
                name.to_string(),
                &msg.author,
                "MessageCommand".to_owned(),
                Box::new(cmd),
            );
        }

        drop(lock);

        Ok(FlopMessagable::Text(format!("Added command `{name}`")))
    }

    fn save(self) -> Vec<u8> {
        Vec::new()
    }
}
