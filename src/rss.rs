use crate::entity::Episode;
use std::error::Error;
use ureq::{Agent, AgentBuilder};

pub type Range = (usize, usize);
pub type NextBytePosition = usize;
pub type EpisodeInfo = (Vec<Episode>, NextBytePosition);
pub type EpisodeResult = Result<EpisodeInfo, Box<dyn Error>>;

pub trait Parser {
    fn parse_rss(&self, url: &str, range: Range) -> EpisodeResult;
}

pub mod sloppy {
    use super::*;
    use sloppy_podcast_tool::{model::Item, parser::Parser as XParser};
    use std::io::BufReader;

    pub struct Client<P: XParser> {
        parser: P,
        agent: Agent,
    }

    impl<P: XParser> Client<P> {
        pub fn new(parser: P) -> Self {
            Client {
                parser,
                agent: AgentBuilder::new().build(),
            }
        }
    }

    impl<P: XParser> Parser for Client<P> {
        fn parse_rss(&self, url: &str, range: Range) -> EpisodeResult {
            let (start, end) = range;
            let rd = self
                .agent
                .get(url)
                .set("Range", &format!("bytes={}-{}", start, end))
                .call()?
                .into_reader();
            let buf_rd = BufReader::new(rd);

            let (items, last_item_position) = self.parser.de_valid(buf_rd)?;

            Ok((
                items
                    .iter()
                    .map(|x: &Item| x.clone().into())
                    .collect::<Vec<Episode>>(),
                start + last_item_position,
            ))
        }
    }
}
