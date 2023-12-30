use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::{
    command::{CmdCtx, FlopMessagable},
    config::Config,
    sql::{CanonicalisedStatus, CmdNode, FlopDB},
    Cli,
};
pub use color_eyre::Result as FlopResult;
use serenity::{async_trait, http::Http, model::prelude::*, prelude::*};
use tokio::time;
use tracing::{debug, error, info};

const FALLBACK_EMOTE: &str = "⚠";
const RESPONSE_CACHE_SIZE: usize = 512;
const ROOT_REGISTRY: &str = "root";

#[derive(Debug)]
#[allow(dead_code)]
pub struct FlopHandler {
    cfg: Config,
    cli: Cli,
    emoji: EmojiCache,
    data: Arc<RwLock<FlopDB>>,
    response_cache: RwLock<HashMap<MessageId, MessageId>>,
}

#[derive(Debug)]
struct EmojiCache {
    emoji: ReactionType,
    text: String,
}

impl FlopHandler {
    pub fn get_db(&self) -> Arc<RwLock<FlopDB>> {
        self.data.clone()
    }

    pub async fn new(cfg: Config, cli: Cli) -> Self {
        // TODO: Move the init stuff to a method taking &mut self
        let emoji = EmojiCache {
            emoji: cfg.emoji.emoji.as_str().try_into().unwrap_or_else(|e| {
                error!("Error constructing reaction emoji:```\n{e}```");
                ReactionType::Unicode(FALLBACK_EMOTE.to_string())
            }),
            text: fomat_reaction_string(&cfg.emoji.phrase),
        };

        let data = Arc::new(RwLock::new(match FlopDB::init(&cli).await {
            Ok(i) => i,
            Err(e) => panic!("Error connstructing database: `{e:?}`"),
        }));

        Self {
            cfg,
            cli,
            emoji,
            data,
            response_cache: RwLock::new(HashMap::with_capacity(RESPONSE_CACHE_SIZE)),
        }
    }

    // Returns the message id of the old response to the message, if there is one
    async fn handle_command(&self, ctx: &Context, msg: Message) -> Option<MessageId> {
        // Check if the messages starts with prefix
        if !msg.author.bot && msg.content.starts_with(&self.cfg.prefix) {
            // Get the name of the command to be ran
            let name = &msg.content[self.cfg.prefix.len()..];
            debug!("command {name} was called");

            // Find the actual command object and obtain a lock for it
            let data_lock = self.data.read().await;
            let canonicalised = data_lock
                .canonicalise_command(ROOT_REGISTRY.to_owned(), name.to_owned())
                .await;

            // Deal with the output of the canonicalisation
            match canonicalised.status {
                CanonicalisedStatus::Success => (),
                CanonicalisedStatus::Overflow => {
                    return self
                        .process_messageable(
                            &msg,
                            FlopMessagable::Text(
                                "This command is nested too deep to be run".to_string(),
                            ),
                            &ctx.http,
                        )
                        .await
                }
                CanonicalisedStatus::NotFound => {
                    if canonicalised.stack.len() > 1 {
                        if let Some((registry, name)) = canonicalised.stack.last() {
                            return self
                                .process_messageable(
                                    &msg,
                                    FlopMessagable::Text(format!(
                                        "Cannot find command {registry}:{name}"
                                    )),
                                    &ctx.http,
                                )
                                .await;
                        }
                    }
                }
                CanonicalisedStatus::Recursive => {
                    let chain = canonicalised
                        .stack
                        .iter()
                        .map(|(r, n)| format!("`{r}:{n}`"))
                        .fold(String::new(), |mut l, r| {
                            l.reserve(r.len() + 4);
                            l.push_str(&r);
                            l.push_str(" -> ");
                            l
                        });
                    return self
                        .process_messageable(
                            &msg,
                            FlopMessagable::Text(format!("Recursive loop:\n{chain}")),
                            &ctx.http,
                        )
                        .await;
                }
                CanonicalisedStatus::FailedSubcommand => {
                    return self
                        .process_messageable(
                            &msg,
                            FlopMessagable::Text(format!(
                                "{0}{1} is a registry, usage `{0}{1} [command name]`",
                                self.cfg.prefix.len(),
                                canonicalised.call
                            )),
                            &ctx.http,
                        )
                        .await
                }
            }

            let (registry, name) = canonicalised
                .stack
                .last()
                .map(|x| x.to_owned())
                .unwrap_or((String::new(), String::new()));

            let Some(entry) = data_lock.get_command(registry.clone(), name.clone()) else {
                error!("Somehow got no responce from a canonicalisaion");
                return None;
            };
            let mut entry = entry.lock().await;
            drop(data_lock);

            // Execute the command
            let cmd_ctx = CmdCtx {
                ctx,
                command: &(self.cfg.prefix.clone() + &canonicalised.call),
                registry: &registry,
                name: &name,
                owner: *entry.get_owner(),
                added: entry.get_added(),
            };
            let node = entry.get_node();

            let CmdNode::Cmd(cmd) = node else {
                error!("Expected a command, not a `{node:?}`!");
                return None;
            };

            let result = cmd.execute(&msg, cmd_ctx, &self.data).await;
            // Drop the lock
            drop(entry);
            // Send the result
            match result {
                Ok(m) => {
                    if !m.is_none() {
                        return self.process_messageable(&msg, m, &ctx.http).await;
                    }
                }
                Err(e) => {
                    error!("Error running ${name} @ `{}`:```rust\n{e}```", msg.link())
                }
            }
        }
        None
    }

    /// Adds a replyed message to the cache
    async fn process_messageable(
        &self,
        source: &Message,
        messagble: FlopMessagable,
        http: &Http,
    ) -> Option<MessageId> {
        let result = messagble.send(source, http).await;
        if let Err(e) = result {
            error!("Error sending reply @ `{}`:```rust\n{e}```", source.link())
        } else if let Ok(reply) = result {
            let mut lock = self.response_cache.write().await;
            let old = lock.insert(source.id, reply.id);
            if lock.len() >= RESPONSE_CACHE_SIZE {
                let mut keys = lock.keys().cloned().collect::<Vec<_>>();
                keys.sort();
                keys.into_iter()
                    .take(lock.len() - RESPONSE_CACHE_SIZE + 1)
                    .for_each(|id| {
                        lock.remove(&id);
                    })
            }
            return old;
        }
        None
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
        self.handle_command(&ctx, msg).await;
    }

    async fn message_update(
        &self,
        ctx: Context,
        _old: Option<Message>,
        new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        debug!("Message updated: {}", event.id);

        // Use the cached message, otherwise fetch the message from discord
        let msg = if let Some(msg) = new {
            msg
        } else {
            match ctx.http.get_message(event.channel_id, event.id).await {
                Ok(msg) => msg,
                Err(e) => {
                    error!(
                        "Error deleting message {}```rust\n{e}```",
                        event.id.link(event.channel_id, event.guild_id)
                    );
                    return;
                }
            }
        };

        // Use the normal message handler, just copy pasted (maybe move to fn)
        if fomat_reaction_string(&msg.content).contains(&self.emoji.text) {
            let result = msg.react(&ctx.http, self.emoji.emoji.clone()).await;
            if let Err(e) = result {
                error!("Error reacting to `{}````rust\n{e}```", msg.link())
            }
        }
        // Handle potental command calls
        if let Some(id) = self.handle_command(&ctx, msg).await {
            if let Err(e) = event.channel_id.delete_message(&ctx.http, id).await {
                error!(
                    "Error deleting message {}```rust\n{e}```",
                    id.link(event.channel_id, event.guild_id)
                )
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }

    async fn message_delete(
        &self,
        ctx: Context,
        channel_id: ChannelId,
        deleted_message_id: MessageId,
        guild_id: Option<GuildId>,
    ) {
        let lock = self.response_cache.read().await;
        if let Some(id) = lock.get(&deleted_message_id) {
            if let Err(e) = channel_id.delete_message(&ctx.http, id).await {
                error!(
                    "Error deleting message {}```rust\n{e}```",
                    id.link(channel_id, guild_id)
                )
            }
        }
    }
}

/// Dedups and removes whitespace from a string
fn fomat_reaction_string(text: &str) -> String {
    let mut chars: Vec<char> = text.chars().filter(|x| !x.is_whitespace()).collect();
    chars.dedup();
    chars.into_iter().collect()
}

/// Function to sync db consistantly
pub async fn db_sync_loop(duration: u64, data: Arc<RwLock<FlopDB>>) {
    let mut interval = time::interval(Duration::from_secs(duration));
    debug!("Started save loop");
    loop {
        interval.tick().await;

        db_sync(data.clone()).await;
    }
}

pub async fn db_sync(data: Arc<RwLock<FlopDB>>) {
    // Get and drain the dirty commands
    let mut lock = data.write().await;
    let dirty = lock.drain_dirty();
    let removed = lock.drain_removed();
    // Drop lock to free db to be used for other purposes
    drop(lock);

    // Get a read lock, as we dont need to write any data for this potentally long running function
    let lock = data.read().await;
    if let Err(e) = lock.sync(dirty, removed).await {
        error!("Error syncing to disk```rust\n{e}```");
    }
}
