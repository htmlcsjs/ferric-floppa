use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};
use serenity::builder::CreateEmbed;
use tracing::{
    callsite::Identifier,
    field::{Field, Visit},
    Level, Metadata, Subscriber,
};
use tracing_subscriber::{layer::Context, Layer};

use crate::Config;

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
        if metadata
            .module_path()
            .is_some_and(|x| !x.starts_with("floppa"))
            || !metadata.target().starts_with("floppa")
        {
            metadata.level() < &Level::DEBUG && metadata.level() < &self.min_level
        } else {
            metadata.level() <= &self.min_level
        }
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
        description = format!("{message}\n{description}");

        let mut embed = CreateEmbed::new()
            .description(description)
            .title(title)
            .field("Level", format!("`{}`", metadata.level()), true)
            .field("Name", format!("`{}`", metadata.name()), false)
            .field("Target", format!("`{}`", metadata.target()), true)
            .color(colour_from_level(*metadata.level()));

        if let Some(path) = metadata.module_path() {
            embed = embed.field("Path", format!("`{path}`"), true);
        }

        if let Some(file) = metadata.file() {
            let mut line_num = "??".to_owned();
            if let Some(num) = metadata.line() {
                line_num = num.to_string();
            }

            embed = embed.field("Location", format!("`{file}:{}`", line_num), false);
        }

        let embed_json = match ureq::serde_json::to_value(embed) {
            Ok(s) => s,
            Err(e) => panic!("Error constructing message: {e}"),
        };

        if let Err(e) = ureq::post(&self.webhook).send_json(WebhookMessage {
            content: None,
            attachments: Vec::new(),
            embeds: vec![embed_json],
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
    embeds: Vec<ureq::serde_json::Value>,
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
