use std::error::Error;

use serenity::prelude::GatewayIntents;

/// This file is used to house constants such as Type definititons and constants
///
/// Nothing dynamic in here please

pub type FlopResult<T> = Result<T, Box<dyn Error + Send + Sync + 'static>>;

#[inline]
pub fn get_intents() -> GatewayIntents {
    GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_MESSAGE_REACTIONS
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::DIRECT_MESSAGE_REACTIONS
        | GatewayIntents::MESSAGE_CONTENT
}
