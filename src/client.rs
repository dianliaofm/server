use crate::{model::Item, parser::Rss, ByteRange};
use simple_error::SimpleError;
use std::error::Error;

pub type NextStart = u32;
pub type FetchInfo = (Vec<Item>, NextStart);
pub type FetchResult = Result<FetchInfo, Box<dyn Error>>;

pub trait Fetcher {
    fn parse_all(&self, url: String) -> Vec<Item>;
    // return next start index
    fn parse_segment(&self, url: String, range: ByteRange, seg_size: u32) -> FetchResult;
}

pub struct Client {
    min_bytes: u32,
}

impl Fetcher for Client {
    fn parse_all(&self, _url: String) -> Vec<Item> {
        unimplemented!()
    }

    fn parse_segment(&self, url: String, range: ByteRange, seg_size: u32) -> FetchResult {
        let (start, end) = range;
        let mut left = start;
        let mut right = start + seg_size;

        let mut list: Vec<Item> = vec![];

        'parse: loop {
            let rss = Rss {
                rss_url: url.clone(),
                range: (left, right),
            };

            let (items, total_bytes) = rss.fetch_items()?;
            list.extend(items);

            left += total_bytes;
            right += total_bytes;

            if total_bytes < self.min_bytes {
                return Err(Box::new(SimpleError::new("segment size not big enough")));
            }

            if right > end {
                break 'parse;
            }
        }

        Ok((list, left))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flexi_logger::Logger;
    use log::debug;

    const TEST_URL: &str = "http://rss.lizhi.fm/rss/14093.xml";

    fn init_log() {
        let _lg = Logger::try_with_str("debug")
            .unwrap()
            .log_to_stdout()
            .start()
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn seg_test1() {
        init_log();
        let client = Client { min_bytes: 10 };
        let range = (3000u32, 20_000u32);
        let size = 1000u32;
        let _ = client
            .parse_segment(TEST_URL.to_string(), range, size)
            .expect("parse segment failed");
    }

    #[test]
    fn seg_test2() {
        init_log();
        let client = Client { min_bytes: 10 };
        let range = (3000u32, 20_000u32);
        let size = 5000u32;
        let (items, next) = client
            .parse_segment(TEST_URL.to_string(), range, size)
            .expect("parse segment failed");
        assert!(items.len() > 1);
        for i in items {
            assert!(i.title.len() > 5);
            debug!("item {}", String::from_utf8_lossy(&i.title));
        }
        debug!("next start byte {}", next);
    }
}
