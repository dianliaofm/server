use crate::{entity::Episode, EpisodeResult, Range};
use ureq::{Agent, AgentBuilder};

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
    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::util::init_log;
        use log::debug;
        use sloppy_podcast_tool::get_parser;

        const TEST_UTL: &str = "http://rss.lizhi.fm/rss/14093.xml";

        #[test]
        fn get_eps() {
            init_log();
            let client = Client::new(get_parser());
            let mut start = 3000usize;
            let win_size = 10_000usize;

            let mut count = 0;
            while count < 2 {
                let (eps, next_start) = client
                    .parse_rss(TEST_UTL, (start, start + win_size))
                    .expect("get eps failed");

                for i in eps {
                    debug!("{:?}", i);
                }
                debug!("next byte start {}", next_start);

                start = next_start;
                count += 1;
            }
        }
    }
}
