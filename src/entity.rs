use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct Episode {
    pub title: String,
    pub subtitle: String,
    pub description: String,
    pub date: String,
    pub timestamp: u64,
    pub url: String,
    pub image: String,
    pub date_key: String,
}
