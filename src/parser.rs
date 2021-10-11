use crate::model::Item;
use quick_xml::{
    events::{BytesText, Event},
    Reader,
};
use std::error::Error;
use std::io::{BufReader, Read};

type ByteRange = (u32, u32);

pub struct Rss {
    pub rss_url: String,
    pub range: ByteRange,
}

impl Rss {
    pub fn fetch(&self) -> Result<impl Read, Box<dyn Error>> {
        let (start, end) = self.range;

        let resp = ureq::get(&self.rss_url)
            .set("Range", &format!("bytes={}-{}", start, end))
            .call()?;
        Ok(resp.into_reader())
    }

    pub fn fetch_items(&self) -> Result<ItemsInfo, Box<dyn Error>> {
        let reader = self.fetch()?;
        Ok(reader_to_xml(reader))
    }
}

const ITEM: &[u8] = b"item";
//const CH: &[u8] = b"channel";
const TITLE: &[u8] = b"title";
const SUB: &[u8] = b"itunes:subtitle";
const PUBDATE: &[u8] = b"pubDate";
const ENCLOSURE: &[u8] = b"enclosure";

type TagStack = Vec<Vec<u8>>;
type ItemsInfo = (Vec<Item>, u32);
// process items and calculate bytes processed
pub fn reader_to_xml(r: impl Read) -> ItemsInfo {
    let buf_rd = BufReader::new(r);

    let mut items = Vec::new();
    let mut current_item = Item::default();
    let mut buf: Vec<u8> = Vec::new();

    let mut total_bytes = 0u32;
    let mut current_bytes = 0u32;

    let mut reader = Reader::from_reader(buf_rd);
    reader.trim_text(true);
    reader.check_end_names(false);

    let mut tag_stack: TagStack = Vec::with_capacity(2);

    loop {
        let mut state: ParseState = ParseState::Empty;
        match reader.read_event(&mut buf) {
            Ok(Event::Eof) => {
                state = ParseState::Eof;
            }
            Err(e) => {
                log::error!("{:?}", e);
            }
            Ok(Event::Start(e)) => {
                let tag = e.name();
                tag_stack.push(tag.to_vec());
            }
            // <enclosure />
            Ok(Event::Empty(e)) => {
                if check_last_item(&tag_stack) {
                    match e.name() {
                        ENCLOSURE => {
                            if let Some(url) =
                                e.attributes().flatten().filter(|x| x.key == b"url").next()
                            {
                                state = ParseState::Enclosure(url.value.to_vec());
                            }
                        }
                        _ => (),
                    }
                }
            }

            Ok(Event::End(e)) => {
                state = ParseState::Empty;
                let last = tag_stack.pop();

                // </item> calculate total bytes; add item to list
                if let Some(t) = last {
                    if t == ITEM && e.name() == ITEM {
                        items.push(current_item.clone());
                        current_item = Item::default();
                        state = ParseState::ItemEnd;
                    }
                }
            }
            Ok(Event::CData(e)) => {
                state = text_state(&state, &e, &tag_stack);
            }
            Ok(Event::Text(e)) => {
                state = text_state(&state, &e, &tag_stack);
            }
            _ => (),
        };

        //buf.len() may not catch plain text at the start of file.
        let pos = reader.buffer_position() as u32;
        let (c, t) = state.calc_bytes(current_bytes, total_bytes, pos);
        current_bytes = c;
        total_bytes = t;
        state.update_item(&mut current_item);
        buf.clear();

        if let ParseState::Eof = state {
            log::debug!("Eof current {} , total {}", current_bytes, total_bytes);
            break;
        }
    }

    (items, total_bytes)
}

fn text_state(_: &ParseState, e: &BytesText, stack: &TagStack) -> ParseState {
    match (check_item(stack, 2), e.unescaped()) {
        (true, Ok(txt)) => {
            let txt_vec = txt.into_owned();
            let tag = stack.last().unwrap();
            match tag.as_slice() {
                TITLE => ParseState::Title(txt_vec),
                SUB => ParseState::Subtitle(txt_vec),
                PUBDATE => ParseState::PubDate(txt_vec),
                _ => ParseState::Empty,
            }
        }
        _ => ParseState::Empty,
    }
}

fn check_item(stack: &TagStack, offset: usize) -> bool {
    let len = stack.len();
    len >= offset && stack[len - offset].as_slice() == ITEM
}

fn check_last_item(stack: &TagStack) -> bool {
    check_item(stack, 1)
}

#[derive(Debug, Clone)]
enum ParseState {
    Empty,
    Title(Vec<u8>),
    Subtitle(Vec<u8>),
    PubDate(Vec<u8>),
    Enclosure(Vec<u8>),
    ItemEnd,
    Eof,
}

impl ParseState {
    fn calc_bytes(&self, _current: u32, total: u32, position: u32) -> (u32, u32) {
        match self {
            Self::ItemEnd => (position, position),
            _ => (position, total),
        }
    }

    fn update_item(&self, item: &mut Item) {
        match self {
            Self::Title(t) => item.title = t.to_vec(),
            Self::Subtitle(t) => item.subtitle = t.to_vec(),
            Self::PubDate(t) => item.pub_date = t.to_vec(),
            Self::Enclosure(t) => item.url = t.to_vec(),
            _ => (),
        }
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
    fn fetch_test() {
        init_log();
        let url = TEST_URL;
        let rss = Rss {
            rss_url: url.to_string(),
            range: (500u32, 8000u32),
        };
        let mut rd = rss.fetch().expect("http request failed");
        let mut buf = [0u8; 1024];
        rd.read(&mut buf).expect("rss response error");
        log::debug!("rss response {}", String::from_utf8_lossy(&buf));
    }

    #[test]
    fn parse_xml() {
        init_log();
        let bytes = std::include_bytes!("../samplerss.xml");
        assert_eq!(bytes.len(), 5114);
        let (items, total) = reader_to_xml(bytes.to_vec().as_slice());
        assert_eq!(items.len(), 6);
        for i in items {
            assert_eq!(i.title.len(), 17);
            assert_eq!(i.pub_date.len(), 30);
            assert_eq!(i.url.len(), 16);
        }
        let last_pos = total as usize;
        //last tag should be </item>
        let delta = 7;
        let item_end = &bytes[(last_pos - delta)..last_pos];
        assert_eq!("</item>".as_bytes(), item_end);
    }

    #[test]
    fn get_items() {
        init_log();
        let client = Rss {
            rss_url: TEST_URL.to_string(),
            range: (1000u32, 10_000u32),
        };
        let (items, total) = client.fetch_items().unwrap();
        debug!("bytes prcoessed {}", total);
        for i in items {
            debug!(
                "{:?}, {:?}",
                String::from_utf8_lossy(&i.title),
                String::from_utf8_lossy(&i.url)
            );
        }
    }
}
