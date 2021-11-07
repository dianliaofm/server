pub mod aws;
pub mod entity;
pub mod rss;
pub mod util;

// default impl of rss parser
pub fn get_parser() -> impl rss::Parser {
    rss::sloppy::Client::new(sloppy_podcast_tool::get_parser())
}
