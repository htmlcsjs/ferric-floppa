use std::{collections::HashMap, fmt::Debug, sync::Arc};

use itertools::Itertools;
use serenity::{
    async_trait,
    model::prelude::{Message, ReactionType, Ready},
    prelude::*,
};
use tracing::{error, info, instrument};

use crate::{
    cfg::FlopConfig,
    command::{DataHolder, FlopCommand},
};

#[derive(Debug)]
pub struct Handler {
    cfg: FlopConfig,
    cache_emoji: ReactionType,
    // TODO: Use something less fucked
    command_map: RwLock<HashMap<String, Arc<dyn FlopCommand>>>,
}

impl Handler {
    pub fn init(cfg: FlopConfig) -> Self {
        let cache_emoji = ReactionType::Custom {
            animated: cfg.emote.animated,
            id: cfg.emote.id.into(),
            name: Some(cfg.emote.name.clone()),
        };

        Self {
            cfg,
            cache_emoji,
            command_map: RwLock::new(HashMap::new()),
        }
    }
    pub async fn add_cmd(&mut self, name: String, cmd: impl FlopCommand + 'static) {
        let mut r = self.command_map.write().await;

        r.insert(name, Arc::new(cmd));
    }
}

#[async_trait]
impl EventHandler for Handler {
    #[instrument(skip_all)]
    async fn message(&self, ctx: Context, msg: Message) {
        if msg
            .content
            .chars()
            .dedup()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
            .contains(&self.cfg.emote.phrase)
        {
            if let Err(e) = msg.react(&ctx.http, self.cache_emoji.clone()).await {
                error!("Error adding reaction: {e:?}");
            }
        }

        if !msg.author.bot && msg.content.chars().next().unwrap_or(' ') == self.cfg.prefix {
            let map_handle = self.command_map.read().await;

            if let Some(cmd) = map_handle
                .get(&msg.content.split(' ').next().unwrap_or_default()[1..].to_lowercase())
            {
                // TODO Error handling logging to discord
                cmd.execute(self, msg, &ctx).await;
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
    //
    // #[instrument(skip_all)]
    // async fn message_update(&self, ctx: Context, msg_update: MessageUpdateEvent) {
    //
    // }
}

#[async_trait]
impl DataHolder for Handler {
    async fn get_cfg(&self) -> FlopConfig {
        self.cfg.clone()
    }

    // TODO sync to FS
    async fn set_cfg(&mut self, new_cfg: FlopConfig) {
        self.cfg = new_cfg;
        self.cache_emoji = ReactionType::Custom {
            animated: self.cfg.emote.animated,
            id: self.cfg.emote.id.into(),
            name: Some(self.cfg.emote.name.clone()),
        };
    }
}
