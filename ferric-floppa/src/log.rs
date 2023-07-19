use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};
use tracing::{
    callsite::Identifier,
    field::{Field, Visit},
    Level, Metadata, Subscriber,
};
use tracing_subscriber::{layer::Context, Layer};
use twilight_model::channel::message::Embed;
use twilight_util::builder::embed::{EmbedBuilder, EmbedFieldBuilder};

use crate::config::Config;

#[derive(Debug)]
pub struct FlopLog {
    min_level: Level,
    webhook_level: Level,
    webhook: String,
}

impl FlopLog {
    pub fn new(cfg: &Config) -> Self {
        Self {
            min_level: level_from_str(&cfg.logging.global_level),
            webhook: cfg.logging.webhook_url.clone(),
            webhook_level: level_from_str(&cfg.logging.webhook_level),
        }
    }
}

impl<S> Layer<S> for FlopLog
where
    S: Subscriber,
{
    fn enabled(&self, metadata: &Metadata, _ctx: Context<S>) -> bool {
        metadata.level() <= &self.min_level
    }

    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        if metadata.level() > &self.webhook_level {
            return;
        }

        let mut visitor = FlopLogVisitor::new();
        event.record(&mut visitor);

        let title = match metadata.module_path() {
            Some(path) => format!("{} in `{path}`", metadata.level()),
            None => metadata.level().to_string(),
        };

        let mut description = String::new();
        let mut message = "No message".to_owned();

        for (_, (name, value)) in visitor.fields {
            if name == "message" {
                message = value.clone();
            } else {
                description += &format!("- `{name}`: `{value}`\n")
            }
        }
        description = format!("# {message}\n{description}");

        let mut embed_builder = EmbedBuilder::new()
            .description(description)
            .title(title)
            .field(EmbedFieldBuilder::new("Level", format!("`{}`", metadata.level())).inline())
            .field(EmbedFieldBuilder::new(
                "Name",
                format!("`{}`", metadata.name()),
            ))
            .field(EmbedFieldBuilder::new("Target", format!("`{}`", metadata.target())).inline())
            .color(colour_from_level(*metadata.level()));

        if let Some(path) = metadata.module_path() {
            embed_builder =
                embed_builder.field(EmbedFieldBuilder::new("Path", format!("`{path}`")).inline());
        }

        if let Some(file) = metadata.file() {
            let mut line_num = "??".to_owned();
            if let Some(num) = metadata.line() {
                line_num = num.to_string();
            }
            embed_builder = embed_builder.field(EmbedFieldBuilder::new(
                "Location",
                format!("`{file}:{}`", line_num),
            ))
        }

        if let Err(e) = ureq::post(&self.webhook).send_json(WebhookMessage {
            content: None,
            attachments: Vec::new(),
            embeds: vec![embed_builder.build()],
        }) {
            panic!("Error sending message to webhook: {e}");
        }
    }
}

#[derive(Debug)]
struct FlopLogVisitor<'a> {
    fields: HashMap<Identifier, (&'a str, String)>,
}

impl<'a> FlopLogVisitor<'a> {
    fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }
}

impl<'a> Visit for FlopLogVisitor<'a> {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.fields
            .insert(field.callsite(), (field.name(), format!("{value:?}")));
    }
}

fn level_from_str(level: &str) -> Level {
    Level::from_str(level).unwrap_or_else(|e| {
        println!("WARNING: Invalid log level id in config `{e}`");
        Level::INFO
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct WebhookMessage {
    content: Option<String>,
    embeds: Vec<Embed>,
    attachments: Vec<String>,
}

fn colour_from_level(level: Level) -> u32 {
    match level {
        Level::ERROR => 0xe06c75,
        Level::WARN => 0xe5c07b,
        Level::INFO => 0x98c379,
        Level::DEBUG => 0x61afef,
        _ => 0x979eab,
    }
}
