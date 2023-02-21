use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlopConfig {
    pub emote: ConfigEmoji,
    pub prefix: char,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigEmoji {
    pub count: u64,
    pub id: u64,
    pub animated: bool,
    pub name: String,
    pub phrase: String,
}
