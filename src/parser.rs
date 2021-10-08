use quick_xml::events::Event;
use quick_xml::Reader;
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

fn reader_to_xml(r: impl Read) {
    let buf_rd = BufReader::new(r);

    let mut reader = Reader::from_reader(buf_rd);
    reader.trim_text(true);

    let mut buf: Vec<u8> = Vec::new();

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Start(ref e)) => {
                println!("event start {:?}", String::from_utf8_lossy(e.name()));
            }
            Ok(Event::Text(e)) => {
                println!("Text {:?}", e);
            }
            Err(e) => {
                println!("error {:?}", e);
            }
            Ok(Event::Eof) => break,
            _ => (),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_test() {
        let url = "http://rss.lizhi.fm/rss/14093.xml";
        let rss = Rss {
            rss_url: url.to_string(),
            range: (0u32, 800u32),
        };
        let rd = rss.fetch();
        reader_to_xml(rd);
    }
}
