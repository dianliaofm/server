use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Item {
    pub title: String,
    #[serde(rename = "itunes:subtitle")]
    pub subtitle: String,
    pub description: String,
    #[serde(rename = "pubDate")]
    pub pub_date: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Enclosure {
    pub url: String,
}
