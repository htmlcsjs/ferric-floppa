use serenity::{builder::CreateMessage, model::prelude::Message};

use crate::consts::FlopResult;

use rand::prelude::SliceRandom;

use super::{DataHolder, FlopCommand};

const NO_CONTENT: &str = "⚠️ This command has no responses";

#[derive(Debug)]
pub struct TextCommand {
    responses: Vec<String>,
}

impl TextCommand {
    pub fn new(responses: Vec<String>) -> Self {
        Self { responses }
    }
}

impl FlopCommand for TextCommand {
    fn execute(
        &self,
        _data: &dyn DataHolder,
        _msg: Message,
        m: &mut CreateMessage,
    ) -> FlopResult<()> {
        let result: Option<&String> = self.responses.choose(&mut rand::thread_rng());
        m.content(result.unwrap_or(&NO_CONTENT.to_string()));
        Ok(())
    }
}
