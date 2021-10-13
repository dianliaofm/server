use sloppy_podcast_tool::{date::to_timestamp, model::Item};

#[derive(Debug, PartialEq, Clone)]
pub struct Episode {
    pub title: String,
    pub subtitle: String,
    pub description: String,
    pub date: String,
    pub timestamp: u64,
    pub url: String,
}

impl From<Item> for Episode {
    fn from(item: Item) -> Self {
        let date = item.pub_date;
        let timestamp = to_timestamp(&date).unwrap_or_default();
        Episode {
            title: item.title,
            subtitle: item.subtitle,
            description: item.description,
            date,
            timestamp,
            url: item.enclosure.url,
        }
    }
}
