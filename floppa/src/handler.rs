use crate::{command::CmdCtx, config::Config, sql::FlopDB, Cli};
pub use color_eyre::Result as FlopResult;
use serenity::{async_trait, model::prelude::*, prelude::*};
use tracing::{debug, error, info};

const FALLBACK_EMOTE: &str = "⚠";

#[derive(Debug)]
#[allow(dead_code)]
pub struct FlopHandler {
    cfg: Config,
    cli: Cli,
    emoji: EmojiCache,
    data: RwLock<FlopDB>,
}

#[derive(Debug)]
struct EmojiCache {
    emoji: ReactionType,
    text: String,
}

impl FlopHandler {
    pub async fn new(cfg: Config, cli: Cli) -> Self {
        // TODO: Move the init stuff to a method taking &mut self
        let emoji = EmojiCache {
            emoji: cfg.emoji.emoji.as_str().try_into().unwrap_or_else(|e| {
                error!("Error constructing reaction emoji:```\n{e}```");
                ReactionType::Unicode(FALLBACK_EMOTE.to_string())
            }),
            text: fomat_reaction_string(&cfg.emoji.phrase),
        };
        let data = match FlopDB::init(&cli).await {
            Ok(i) => i,
            Err(e) => panic!("Error connstructing database: `{e:?}`"),
        };

        Self {
            cfg,
            cli,
            emoji,
            data: RwLock::new(data),
        }
    }

    async fn handle_command(&self, ctx: &Context, msg: Message) {
        if !msg.author.bot && msg.content.starts_with(&self.cfg.prefix) {
            // Check if the messages starts with prefix, and if so,
            // get the first "word"
            let Some(s) = msg.content.split_whitespace().next() else {
                return;
            };

            // Get the name of the command to be ran
            let name = &s[self.cfg.prefix.len()..];
            debug!("command {name} was called");

            // Find the actual command object and obtain a lock for it
            // TODO write a symlink algo
            let data_lock = self.data.read().await;
            let Some(cmd) = data_lock.get_command("root".to_owned(), name.to_owned()) else {
                return;
            };
            let mut cmd = cmd.lock().await;
            drop(data_lock);

            // Execute the command
            let cmd_ctx = CmdCtx {
                ctx,
                command: s,
                registry: "root",
                name,
                owner: *cmd.get_owner(),
                added: cmd.get_added(),
            };
            let result = cmd.get_inner().execute(&msg, cmd_ctx, &self.data).await;
            // Drop the lock
            drop(cmd);
            // Send the result
            match result {
                Ok(m) => {
                    if !m.is_none() {
                        if let Err(e) = m.send(&msg, &ctx.http).await {
                            error!("Error sending ${name} @ `{}`:```rust\n{e}```", msg.link())
                        }
                    }
                }
                Err(e) => {
                    error!("Error running ${name} @ `{}`:```rust\n{e}```", msg.link())
                }
            }
        }
    }
}

#[async_trait]
impl EventHandler for FlopHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        if fomat_reaction_string(&msg.content).contains(&self.emoji.text) {
            let result = msg.react(&ctx.http, self.emoji.emoji.clone()).await;
            if let Err(e) = result {
                error!("Error reacting to `{}`:`{e}`", msg.link())
            }
        }
        // Handle potental command calls
        self.handle_command(&ctx, msg).await
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }
}

/// Dedups and removes whitespace from a string
fn fomat_reaction_string(text: &str) -> String {
    let mut chars: Vec<char> = text.chars().filter(|x| !x.is_whitespace()).collect();
    chars.dedup();
    chars.into_iter().collect()
}
