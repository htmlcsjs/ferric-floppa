use std::fmt::Debug;

use messagable::Messagable;
use serenity::{
    async_trait,
    builder::{CreateAllowedMentions, CreateEmbed, CreateMessage},
    http::Http,
    model::prelude::*,
    prelude::*,
};

use crate::{sql::FlopDB, Cli, FlopResult};

#[async_trait]
pub trait Command: Debug {
    /// Constructs the command from CLI options and config, and any data serialised to disk
    fn construct(cli: &Cli, data: &[u8]) -> FlopResult<Self>
    where
        Self: Sized;

    /// Executes the command on the given Message event
    async fn execute<'a>(&mut self, event: &Message, ctx: CmdCtx<'a>)
        -> FlopResult<FlopMessagable>;

    /// Allows the command to serialise data to be asked
    /// Consumes the command, so it will be reinitalised
    fn save(self) -> Vec<u8>;

    // Gets the raw form of the Command
    // TODO: epic macro to sealise src code at compile time
    //fn raw(&self) -> &str;
}

#[async_trait]
/// Extended form of [`Command`] that gives access to more stuff when ran
pub trait ExtendedCommand: Debug {
    /// Constructs the command from CLI options and config, and any data serialised to disk
    fn construct(cli: &Cli, data: &[u8]) -> FlopResult<Self>
    where
        Self: Sized;

    /// Executes the command on the given Message event
    async fn execute<'a>(
        &mut self,
        event: &Message,
        ctx: CmdCtx<'a>,
        data: &RwLock<FlopDB>,
    ) -> FlopResult<FlopMessagable>;

    /// Allows the command to serialise data to be asked
    /// Consumes the command, so it will be reinitalised
    fn save(self) -> Vec<u8>;

    // Gets the raw form of the Command
    // TODO: epic macro to sealise src code at compile time
    //fn raw(&self) -> &str;
}

#[async_trait]
impl<T> ExtendedCommand for T
where
    T: Command + Send + Sync,
{
    /// Constructs the command from CLI options and config, and any data serialised to disk
    fn construct(cli: &Cli, data: &[u8]) -> FlopResult<Self>
    where
        Self: Sized,
    {
        <Self as Command>::construct(cli, data)
    }

    /// Executes the command on the given Message event
    async fn execute<'a>(
        &mut self,
        event: &Message,
        ctx: CmdCtx<'a>,
        _data: &RwLock<FlopDB>,
    ) -> FlopResult<FlopMessagable> {
        <Self as Command>::execute(self, event, ctx).await
    }

    /// Allows the command to serialise data to be asked
    /// Consumes the command, so it will be reinitalised
    fn save(self) -> Vec<u8> {
        <Self as Command>::save(self)
    }

    // Gets the raw form of the Command
    // TODO: epic macro to sealise src code at compile time
    //fn raw(&self) -> &str;
}

#[derive(Debug)]
/// Provided Extra context to commands, like if they were ran under an alias
pub struct CmdCtx<'a> {
    pub ctx: &'a Context,
    pub command: &'a str,
    pub registry: &'a str,
    pub name: &'a str,
    pub owner: UserId,
    pub added: i64,
}

/// Enum for return values of [`Command::execute`]
#[derive(Debug, Clone)]
pub enum FlopMessagable {
    /// Sets the message body text
    Text(String),
    /// Sends the list of embeds
    Embeds(Vec<CreateEmbed>),
    /// Stops the response from ping people and replies to the sender
    Response(MessageReference),
    /// There should be no reply from the bot
    None,
}

impl Messagable for FlopMessagable {
    fn modify_message(self, builder: CreateMessage) -> CreateMessage {
        match self {
            FlopMessagable::Text(s) => s.modify_message(builder),
            FlopMessagable::Embeds(e) => e.modify_message(builder),
            FlopMessagable::Response(msg) => builder
                .allowed_mentions(CreateAllowedMentions::new().replied_user(false))
                .reference_message(msg),
            FlopMessagable::None => builder,
        }
    }
}

impl FlopMessagable {
    pub async fn send(self, msg: &Message, http: &Http) -> FlopResult<Message> {
        let chain = self.chain(FlopMessagable::Response(msg.into()));
        Ok(msg
            .channel_id
            .send_message(http, chain.apply_default())
            .await?)
    }

    pub const fn is_none(&self) -> bool {
        matches!(self, FlopMessagable::None)
    }
}

impl From<String> for FlopMessagable {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<CreateEmbed> for FlopMessagable {
    fn from(value: CreateEmbed) -> Self {
        FlopMessagable::Embeds(vec![value])
    }
}

impl From<Vec<CreateEmbed>> for FlopMessagable {
    fn from(value: Vec<CreateEmbed>) -> Self {
        FlopMessagable::Embeds(value)
    }
}

impl From<&Message> for FlopMessagable {
    fn from(msg: &Message) -> Self {
        FlopMessagable::Response(msg.into())
    }
}

const OTHER_CHARS: [char; 2] = ['_', '-'];
pub fn check_name(name: &str) -> bool {
    name.chars()
        .all(|x| x.is_alphanumeric() || OTHER_CHARS.contains(&x))
}
