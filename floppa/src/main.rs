mod command;
pub mod config;
mod log;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process,
    sync::Arc,
};

use clap::Parser;
pub use color_eyre::Result as FlopResult;
use command::{Command, MessageCommand};
use config::Config;
use log::FlopLog;
use serenity::{async_trait, model::prelude::*, prelude::*};
use tokio::{fs, sync::RwLock};
use tracing::{debug, error, info};
use tracing_subscriber::prelude::*;

const FALLBACK_EMOTE: &str = "⚠";

#[tokio::main]
async fn main() {
    color_eyre::install().expect("failed to install panic handler");

    let cli = match Cli::initlise() {
        Ok(inner) => inner,
        Err(e) => panic!("Fatal error loading cli args:\n{e}"),
    };

    let cfg = match Config::load_from_fs(&cli) {
        Ok(inner) => inner,
        Err(e) => panic!("Fatal error during initial config loading:\n{e}"),
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(FlopLog::new(&cfg))
        .init();

    if let Err(e) = run(cli, cfg).await {
        error!("Fatal error encountered: {e}")
    }
}

async fn run(cli: Cli, cfg: Config) -> FlopResult<()> {
    // TODO: Have a default for this
    // let _cfg = Arc::new(RwLock::new(cfg));

    let temp_token = fs::read_to_string(cli.get_path("token")).await?;
    let token = temp_token.trim().to_string();

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(FlopHandler::new(cfg, cli))
        .await?;

    client.start().await?;

    Ok(())
}

pub type ThreadCfg = Arc<RwLock<Config>>;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(
        short,
        long,
        value_name = "PATH",
        default_value = ".",
        hide_default_value = true
    )]
    /// Sets the directory to be used as the base at runtime.
    /// Default is the current working directory
    run_dir: PathBuf,
}

impl Cli {
    pub fn initlise() -> FlopResult<Self> {
        let mut new = Self::parse();
        new.run_dir = new.run_dir.canonicalize()?;
        if !new.run_dir.is_dir() {
            error!("{} is not a directory!", new.run_dir.display());
            process::exit(1);
        }
        Ok(new)
    }

    #[inline]
    pub fn get_path(&self, path: impl AsRef<Path>) -> PathBuf {
        self.run_dir.join(path)
    }
}

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
                &new.cfg,
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
