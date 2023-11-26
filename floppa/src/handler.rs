use std::{collections::HashMap, sync::Arc};

use crate::{
    command::{Command, MessageCommand},
    config::Config,
    Cli,
};
pub use color_eyre::Result as FlopResult;
use serenity::{async_trait, model::prelude::*, prelude::*};
use tracing::{debug, error, info};

const FALLBACK_EMOTE: &str = "⚠";

#[derive(Debug)]
pub struct FlopHandler {
    cfg: Config,
    cli: Cli,
    emoji: EmojiCache,
    // TODO: improve cmd registry
    commands: HashMap<String, CommandsValue>,
}

#[derive(Debug)]
struct EmojiCache {
    emoji: ReactionType,
    text: String,
}

type CommandsValue = Arc<Mutex<dyn Command + Send + Sync>>;

impl FlopHandler {
    pub fn new(cfg: Config, cli: Cli) -> Self {
        // TODO: Move the init stuff to a method taking &mut self
        let emoji = EmojiCache {
            emoji: cfg.emoji.emoji.as_str().try_into().unwrap_or_else(|e| {
                error!("Error constructing reaction emoji:```\n{e}```");
                ReactionType::Unicode(FALLBACK_EMOTE.to_string())
            }),
            text: fomat_reaction_string(&cfg.emoji.phrase),
        };
        let mut new = Self {
            cfg,
            cli,
            emoji,
            commands: HashMap::new(),
        };
        new.commands.insert(
            "halp".to_string(),
            Arc::new(Mutex::new(MessageCommand::construct(
                &new.cli,
                "flop is dead".into(),
            ))),
        );
        new
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
            let cmds = &self.commands;
            debug!("command {} was called", name);

            // Find the actual command object and obtain a lock for it
            let Some(cmd) = cmds.get(name) else {
                return;
            };
            let mut cmd = cmd.lock().await;

            // Execute the command
            let result = cmd.execute(&msg, ctx).await;
            // Send the result
            match result {
                Ok(Some(m)) => {
                    if let Err(e) = m.send(&msg, &ctx.http).await {
                        error!("Error sending ${name} @ `{}`:```rust\n{e}```", msg.link())
                    }
                }
                Err(e) => {
                    error!("Error running ${name} @ `{}`:```rust\n{e}```", msg.link())
                }
                _ => (),
            }
            // Drop the lock
            drop(cmd);
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
        // Handle potental command calls.
        self.handle_command(&ctx, msg).await;
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
