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

const ITEM: &[u8] = b"item";
//const CH: &[u8] = b"channel";
const TITLE: &[u8] = b"title";
const SUB: &[u8] = b"itunes:subtitle";
const PUBDATE: &[u8] = b"pubDate";

// process items and calculate bytes processed
fn reader_to_xml(r: impl Read) -> (Vec<Item>, u32) {
    let buf_rd = BufReader::new(r);

    let mut items = Vec::new();
    let mut current_item = Item::default();
    let mut buf: Vec<u8> = Vec::new();

    let mut total_bytes = 0u32;
    let mut current_bytes = 0u32;

    let mut reader = Reader::from_reader(buf_rd);
    reader.trim_text(true);
    //reader.check_end_names(false);

    let mut tag_stack: Vec<Vec<u8>> = Vec::with_capacity(2);
    let mut state: ParseState = ParseState::Empty;

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Eof) => {
                log::debug!("Eof break");
                break;
            }
            Err(e) => {
                log::error!("{:?}", e);
            }
            Ok(Event::Start(e)) => {
                let tag = e.name();
                let len = tag_stack.len();
                if len > 0 && tag_stack[len - 1].as_slice() == ITEM {
                    match tag {
                        TITLE => state = ParseState::Title(vec![]),
                        SUB => state = ParseState::Subtitle(vec![]),
                        PUBDATE => state = ParseState::PubDate(vec![]),
                        _ => (),
                    }
                }
                tag_stack.push(tag.to_vec());
            }
            Ok(Event::End(e)) => {
                state = ParseState::Empty;
                tag_stack.pop();

                if e.name() == ITEM {
                    items.push(current_item.clone());
                    current_item = Item::default();
                }
            }
            Ok(Event::CData(e)) => {
                if let Ok(t) = e.unescaped() {
                    state.set_text(t.to_vec());
                }
            }
            Ok(Event::Text(e)) => {
                if let Ok(t) = e.unescaped() {
                    state.set_text(t.to_vec());
                }
            }
            _ => (),
        };
        let len = buf.len() as u32;
        let (c, t) = state.calc_bytes(current_bytes, total_bytes, len);
        current_bytes = c;
        total_bytes = t;
        state.update_item(&mut current_item);
        buf.clear();
    }

    (items, total_bytes)
}

#[derive(Debug, Clone)]
enum ParseState {
    Empty,
    Title(Vec<u8>),
    Subtitle(Vec<u8>),
    PubDate(Vec<u8>),
    Enclosure,
}
impl ParseState {
    fn calc_bytes(&self, current: u32, total: u32, len: u32) -> (u32, u32) {
        match self {
            _ => (current + len, total),
        }
    }

    fn update_item(&self, item: &mut Item) {
        match self {
            Self::Title(t) => item.title = String::from_utf8(t.to_vec()).unwrap(),
            Self::Subtitle(t) => item.subtitle = String::from_utf8(t.to_vec()).unwrap(),
            Self::PubDate(t) => item.pub_date = String::from_utf8(t.to_vec()).unwrap(),
            _ => (),
        }
    }
    fn set_text(&mut self, text: Vec<u8>) {
        match self {
            Self::Title(xs) => *xs = text,
            Self::Subtitle(xs) => *xs = text,
            Self::PubDate(xs) => *xs = text,
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
            range: (500u32, 8000u32),
        };
        let mut rd = rss.fetch();
        let mut buf = [0u8; 1024];
        rd.read(&mut buf).expect("rss response error");
        log::debug!("rss response {}", String::from_utf8_lossy(&buf));
    }

    #[test]
    fn parse_xml() {
        let _lg = Logger::try_with_str("debug")
            .unwrap()
            .log_to_stdout()
            .start()
            .unwrap();
        let bytes = std::include_bytes!("../samplerss.xml");
        let (items, total) = reader_to_xml(bytes.to_vec().as_slice());
        for (n, i) in items.iter().enumerate() {
            log::debug!("items {} {:?}, ", n, i);
        }
        log::debug!("total bytes {:?}", total);
    }
}
