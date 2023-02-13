#[derive(Debug)]
pub struct Handler {
    cfg: Config,
    cache_emoji: ReactionType,
}

impl Handler {
    fn init(cfg: Config) -> Self {
        let cache_emoji = ReactionType::Custom {
            animated: cfg.emote.animated,
            id: cfg.emote.id.into(),
            name: Some(cfg.emote.name.clone()),
        };

        Self { cfg, cache_emoji }
    }
}

#[async_trait]
impl EventHandler for Handler {
    #[instrument(skip_all)]
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!amogus" && !msg.author.bot {
            if let Err(e) = msg.reply(&ctx.http, "imposterious").await {
                error!("Error sending message: {e:?}");
            }
        }
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
    }

    #[instrument(skip_all)]
    async fn ready(&self, _ctx: Context, ready: Ready) {
        info!(
            "Logged in as {}#{:04}",
            ready.user.name, ready.user.discriminator
        );
    }
}
