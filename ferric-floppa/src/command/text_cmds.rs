use std::time::Duration;

use serenity::{async_trait, model::prelude::Message, prelude::Context};

use rand::prelude::SliceRandom;

use crate::{send_reply_text, util::send_msg};

use super::{DataHolder, FlopCommand};

const NO_CONTENT: &str = "⚠️ This command has no responses";

#[derive(Debug)]
pub struct TextCmd {
    responses: Vec<String>,
}

impl TextCmd {
    pub fn new(responses: Vec<String>) -> Self {
        Self { responses }
    }
}

#[async_trait]
impl FlopCommand for TextCmd {
    async fn execute(&self, _data: &dyn DataHolder, msg: Message, ctx: &Context) {
        let result: Option<&String> = self.responses.choose(&mut rand::thread_rng());
        send_reply_text!(result.unwrap_or(&NO_CONTENT.to_string()), ctx, msg)
    }
}

#[derive(Debug)]
pub struct SleepCmd;

#[async_trait]
impl FlopCommand for SleepCmd {
    async fn execute(&self, _data: &dyn DataHolder, msg: Message, ctx: &Context) {
        tokio::time::sleep(Duration::from_secs(1)).await;
        send_reply_text!("Epic sleep", ctx, msg)
    }
}
