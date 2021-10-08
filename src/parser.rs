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
    let mut flush_bytes = false;

    let mut reader = Reader::from_reader(buf_rd);
    reader.trim_text(true);

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                //println!("event start {}", String::from_utf8_lossy(e.name()));
            }
            Ok(Event::End(ref e)) => {
                if e.name() == b"item" {
                    log::debug!("item end current {}, total {}", current_bytes, total_bytes);
                    flush_bytes = true;
                }
            }
            Ok(Event::Eof) => break,
            _ => (),
        }
        current_bytes += buf.len() as u32;
        if flush_bytes {
            total_bytes += current_bytes;
            current_bytes = 0;
            flush_bytes = false;
        }
        buf.clear();
    }

    (items, total_bytes)
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
