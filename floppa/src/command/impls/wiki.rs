use serenity::{async_trait, model::prelude::Message};
use tracing::error;

use crate::{
    command::{inner::CmdCtx, Command, FlopMessagable},
    Cli, FlopResult,
};

const ENDPOINT: &str =
    "https://en.wikipedia.org/w/api.php?action=query&format=json&list=search&srsearch=";
const WIKI_PAGE: &str = "https://en.wikipedia.org/wiki/";
const NO_PAGE_MSG: &str = "Could not find page.";

#[derive(Debug)]
pub struct WikiCommand;

#[async_trait]
impl Command for WikiCommand {
    fn construct(_cli: &Cli, _data: &[u8]) -> FlopResult<Self> {
        Ok(Self)
    }

    async fn execute<'a>(&mut self, msg: &Message, ctx: CmdCtx<'a>) -> FlopResult<FlopMessagable> {
        // shittly URLEncode the message
        let args = msg
            .content
            .trim_start_matches(ctx.command)
            .trim()
            .replace(char::is_whitespace, "+");
        if args.is_empty() {
            return Ok(FlopMessagable::Text(
                "Missing argument for wiki lookup".to_owned(),
            ));
        }

        // make a request to Wikipedia
        let data_result = reqwest::get(format!("{ENDPOINT}{args}"))
            .await?
            .json::<serde_json::Value>()
            .await;

        let data = match data_result {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to decode response: `{e}`");
                return Ok(FlopMessagable::Text(format!(
                    "Failed to decode response from Wikipedia:\n```{e}```"
                )));
            }
        };

        // Extract the title and send response
        if let Some(title) = data["query"]["search"][0]["title"].as_str() {
            Ok(FlopMessagable::Text(format!("{WIKI_PAGE}{title}")))
        } else {
            Ok(FlopMessagable::Text(NO_PAGE_MSG.to_string()))
        }
    }

    fn save(&self) -> Option<Vec<u8>> {
        None
    }
}
