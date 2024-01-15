use serenity::all::{Mention, UserId};
use tracing::error;

use std::str::FromStr;

pub fn try_get_user(text: &str) -> Option<UserId> {
    if text.chars().all(|c| c.is_ascii_digit()) {
        let id = match text.parse::<u64>() {
            Ok(id) => id,
            Err(e) => {
                error!("Unexpected error parsing number: {e}");
                return None;
            }
        };
        return Some(UserId::new(id));
    }

    match Mention::from_str(text) {
        Ok(Mention::User(id)) => Some(id),
        _ => None,
    }
}
