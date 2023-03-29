use std::{collections::HashMap, fmt::Debug, sync::Arc};

use itertools::Itertools;
use serenity::{
    async_trait,
    model::prelude::{EmojiId, Message, MessageUpdateEvent, Ready},
    prelude::*,
};
use tracing::{error, info, instrument, warn};

use crate::{cfg::FlopConfig, command::FlopCommand, util::error::report_error};

// maybe want to put data into ctx?
#[derive(Debug)]
pub struct Handler {
    cache: Option<HandlerCache>,
    // TODO: Use something less fucked
    command_map: RwLock<HashMap<String, Arc<dyn FlopCommand>>>,
}

impl Handler {
    pub fn new() -> Self {
        Self {
            cache: None,
            command_map: RwLock::new(HashMap::new()),
        }
    }

    pub async fn add_cmd(&mut self, name: String, cmd: impl FlopCommand + 'static) {
        let mut map = self.command_map.write().await;
        map.insert(name, Arc::new(cmd));
    }

    pub async fn update_cache(&mut self, ctx: &Context) {
        #[allow(clippy::let_and_return)] // Otherwise data_r wouldn't live long enough
        let cfg = {
            let data_r = ctx.data.read().await;
            let cfg_lock = data_r.get::<DataHolderKey>().expect("Expected Dataholder");
            let cfg = cfg_lock.read().await.get_cfg();
            cfg
        };
        self.cache = Some(HandlerCache {
            emoji: EmojiId::from(cfg.emote.id),

            phrase: cfg.emote.phrase,
            prefix: cfg.prefix,
        });
        ctx.cache.set_max_messages(cfg.message_cache_size);
    }
}

#[async_trait]
impl EventHandler for Handler {
    #[instrument(skip_all)]
    async fn message(&self, ctx: Context, msg: Message) {
        // might want to cahce stuff
        // TODO: Split this off into a seperate function, for code cleniness
        if let Some(cache) = &self.cache {
            if msg
                .content
                .chars()
                .dedup()
                .filter(|c| !c.is_whitespace())
                .collect::<String>()
                .contains(&cache.phrase)
            {
                if let Err(e) = msg.react(&ctx.http, cache.emoji).await {
                    error!("Error adding reaction: {e:?}");
                }
            }

            if !msg.author.bot && msg.content.chars().next().unwrap_or(' ') == cache.prefix {
                let map_handle = self.command_map.read().await.clone();
                let name = msg.content.split(' ').next().unwrap_or_default()[1..].to_lowercase();

                if let Some(cmd) = map_handle.get(&name) {
                    let typing = msg.channel_id.start_typing(&ctx.http);
                    cmd.execute(msg, &ctx).await;
                    match typing {
                        Ok(ty) => ty.stop().unwrap(),
                        Err(e) => report_error(Box::new(e), "Error creating typing callback"),
                    }
                }
            }
        }
    }

    #[instrument(skip_all)]
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!(
            "Logged in as {}#{:04}",
            ready.user.name, ready.user.discriminator
        );
    }

    #[instrument(skip_all)]
    async fn message_update(
        &self,
        ctx: Context,
        _old: Option<Message>,
        new: Option<Message>,
        event: MessageUpdateEvent,
    ) {
        // TODO compare old message content and new message content and react apporiatly
        if let Some(msg) = new {
            self.message(ctx, msg).await
        } else {
            warn!("Could not get the new message object for {:?}", event)
        }
    }
}

#[derive(Debug)]
pub struct DataHolder {
    cfg: FlopConfig,
}

impl DataHolder {
    pub fn get_cfg(&self) -> FlopConfig {
        self.cfg.clone()
    }

    // TODO sync to FS
    pub async fn set_cfg(&mut self, new_cfg: FlopConfig) {
        self.cfg = new_cfg;
    }

    pub fn new(cfg: FlopConfig) -> Self {
        Self { cfg }
    }
}

pub struct DataHolderKey;

impl TypeMapKey for DataHolderKey {
    type Value = Arc<RwLock<DataHolder>>;
}

#[derive(Debug)]
struct HandlerCache {
    emoji: EmojiId,
    phrase: String,
    prefix: char,
}
