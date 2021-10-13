pub mod aws;
pub mod entity;
pub mod rss;
pub mod util;

use entity::Episode;
use std::error::Error;
pub type Range = (usize, usize);
pub type NextBytePosition = usize;
pub type EpisodeInfo = (Vec<Episode>, NextBytePosition);
pub type EpisodeResult = Result<EpisodeInfo, Box<dyn Error>>;

// default impl of rss parser
pub fn get_parser() -> impl rss::Parser {
    rss::sloppy::Client::new(sloppy_podcast_tool::get_parser())
}
