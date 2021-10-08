#[derive(Debug, Clone, PartialEq, Default)]
pub struct Item {
    pub title: String,
    pub subtitle: String,
    pub pub_date: String,
    pub url: String,
    pub length: u32,
}
