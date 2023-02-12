use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub emote: ConfigEmoji,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigEmoji {
    pub count: u64,
    pub id: u64,
    pub animated: bool,
    pub name: String,
    pub phrase: String,
}
