use crate::model::Item;
use quick_xml::{events::Event, Reader};
use std::io::{BufReader, Read};

type ByteRange = (u32, u32);

pub struct Rss {
    rss_url: String,
    range: ByteRange,
}

impl Rss {
    pub fn fetch(&self) -> impl Read {
        let (start, end) = self.range;

        ureq::get(&self.rss_url)
            .set("Range", &format!("bytes={}-{}", start, end))
            .call()
            .unwrap()
            .into_reader()
    }
}

// process items and calculate bytes processed
fn reader_to_xml(r: impl Read) -> (Vec<Item>, u32) {
    let buf_rd = BufReader::new(r);

    let mut items = Vec::new();
    let mut current: Item = Item::default();
    let mut buf = Vec::new();

    let mut total_bytes = 0u32;
    let mut current_bytes = 0u32;

    let mut reader = Reader::from_reader(buf_rd);
    reader.trim_text(true);

    let mut state: Option<ParseState> = None;

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Eof) => break,
            Err(e) => log::error!("{:?}", e),
            Ok(ev) => {}
        }
        state.iter().for_each(|st| {
            let (c, t) = st.calc_bytes(current_bytes, total_bytes);
            current_bytes = c;
            total_bytes = t;
            st.make_item(&mut current, &mut items);
        });
        buf.clear();
    }

    (items, total_bytes)
}

enum ParseState {
    ItemStart,
    ItemEnd,
    Title,
    Subtitle,
    PubDate,
    Enclosure,
}
impl ParseState {
    fn calc_bytes(&self, current: u32, total: u32) -> (u32, u32) {
        match self {
            _ => (current, total),
        }
    }

    fn make_item(&self, item: &mut Item, items: &mut Vec<Item>)  {
        match self {
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flexi_logger::Logger;

    #[test]
    fn fetch_test() {
        let _lg = Logger::try_with_str("debug")
            .unwrap()
            .log_to_stdout()
            .start()
            .unwrap();
        let url = "http://rss.lizhi.fm/rss/14093.xml";
        let rss = Rss {
            rss_url: url.to_string(),
            range: (0u32, 8000u32),
        };
        let rd = rss.fetch();
        let (_, len) = reader_to_xml(rd);
        log::debug!("bytes processed {}", len);
    }
}
