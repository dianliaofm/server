#[cfg(test)]
mod tests {
    //use super::*;
    use crate::util::init_log;
    use log::debug;
    use sloppy_podcast_tool::parser::{quick::Client, Parser};
    use std::io::BufReader;

    const TEST_UTL: &str = "http://rss.lizhi.fm/rss/14093.xml";

    #[test]
    fn check_rss() {
        init_log();
        let start = 3000usize;
        let end = 20000usize;
        let rd = ureq::get(TEST_UTL)
            .set("Range", &format!("bytes={}-{}", start, end))
            .call()
            .expect("http failed")
            .into_reader();
        let bufrd = BufReader::new(rd);
        let client = Client {};
        let (items, right) = client.de_valid(bufrd).expect("items failed");
        for i in items {
            debug!("{}", i.title);
        }
        debug!("next start {}", right);
    }
}
