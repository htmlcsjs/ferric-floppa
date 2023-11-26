use std::fmt::Debug;

use messagable::Messagable;
use serenity::{
    async_trait,
    builder::{CreateEmbed, CreateMessage},
    http::Http,
    model::prelude::*,
    prelude::*,
};

use crate::{Cli, FlopResult};

#[async_trait]
pub trait Command: Debug {
    /// Constructs the command from CLI options and config, and any data serialised to disk
    fn construct(cli: &Cli, data: rmpv::Value) -> Self
    where
        Self: Sized;

    /// Executes the command on the given Message event
    async fn execute(
        &mut self,
        event: &Message,
        ctx: &Context,
    ) -> FlopResult<Option<FlopMessagable>>;

    /// Allows the command to serialise data to be asked
    /// Consumes the command, so it will be reinitalised
    fn save(self) -> rmpv::Value;

    /// Gets the raw form of the Command
    // TODO: epic macro to sealise src code at compile time
    fn raw(&self) -> &str;
}

/// Enum for return values of [`Command::execute`]
#[derive(Debug, Clone)]
pub enum FlopMessagable<'a> {
    /// Sets the message body text
    Text(String),
    /// Sends the list of embeds
    Embeds(Vec<CreateEmbed>),
    /// Stops the response from ping people and replies to the sender
    Response(&'a Message),
}

impl Messagable for FlopMessagable<'_> {
    fn modify_message<'a, 'b>(
        self,
        builder: &'a mut CreateMessage<'b>,
    ) -> &'a mut CreateMessage<'b> {
        match self {
            FlopMessagable::Text(s) => s.modify_message(builder),
            FlopMessagable::Embeds(e) => e.modify_message(builder),
            FlopMessagable::Response(msg) => builder
                .allowed_mentions(|x| x.empty_parse().replied_user(false))
                .reference_message(msg),
        }
    }
}

impl FlopMessagable<'_> {
    pub async fn send(self, msg: &Message, http: &Http) -> FlopResult<Message> {
        let chain = self.chain(FlopMessagable::Response(msg));
        Ok(msg
            .channel_id
            .send_message(http, |x| chain.modify_message(x))
            .await?)
    }
}

impl From<String> for FlopMessagable<'_> {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<CreateEmbed> for FlopMessagable<'_> {
    fn from(value: CreateEmbed) -> Self {
        FlopMessagable::Embeds(vec![value])
    }
}

impl From<Vec<CreateEmbed>> for FlopMessagable<'_> {
    fn from(value: Vec<CreateEmbed>) -> Self {
        FlopMessagable::Embeds(value)
    }
}

impl<'a> From<&'a Message> for FlopMessagable<'a> {
    fn from(value: &'a Message) -> Self {
        FlopMessagable::Response(value)
    }
}
