use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlopConfig {
    #[serde(default)]
    pub emote: ConfigEmoji,
    pub prefix: char,
    pub message_cache_size: usize,
    pub error_channel: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigEmoji {
    pub count: u64,
    pub id: u64,
    pub animated: bool,
    pub name: String,
    pub phrase: String,
}

impl Default for ConfigEmoji {
    fn default() -> Self {
        Self {
            count: 0,
            id: 853358698964713523,
            animated: false,
            name: "flop".to_string(),
            phrase: "flop".to_string(),
        }
    }
}
