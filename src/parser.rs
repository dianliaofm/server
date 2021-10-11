use std::io::BufRead;

pub type ItemStart = u32;
pub type ItemEnd = u32;
pub type ValidItems = (Vec<u8>, ItemStart, ItemEnd);
pub trait Parser {
    fn parse_valid(&self, input: impl BufRead) -> ValidItems;
}

pub mod quick {
    use super::*;
    use quick_xml::events::Event;
    use quick_xml::Reader;
    use quick_xml::Writer;
    use std::io::Cursor;

    const ITEM: &[u8] = b"item";

    pub struct Client {}

    impl Parser for Client {
        fn parse_valid(&self, input: impl BufRead) -> ValidItems {
            let mut reader = Reader::from_reader(input);
            reader.trim_text(true);
            reader.check_end_names(false);
            let mut writer = Writer::new(Cursor::new(Vec::new()));
            let mut buf: Vec<u8> = Vec::new();

            let mut left: Option<u32> = None;
            let mut right = 0u32;

            loop {
                match reader.read_event(&mut buf) {
                    Ok(Event::Eof) => break,
                    Ok(Event::Start(e)) if e.name() == ITEM => {
                        if let None = left {
                            left = Some(reader.buffer_position() as u32);
                        }
                        assert!(writer.write_event(Event::Start(e.into_owned())).is_ok());
                    }
                    Ok(Event::End(e)) if e.name() == ITEM => {
                        right = reader.buffer_position() as u32;
                    }
                    Ok(e) => assert!(writer.write_event(e).is_ok()),
                    Err(e) => log::debug!("Error at {}: {:?}", reader.buffer_position(), e),
                }
            }

            (
                writer.into_inner().into_inner(),
                left.unwrap_or_default(),
                right,
            )
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::util::init_log;
        use log::debug;
        use std::include_bytes;

        #[test]
        fn xml_valid() {
            init_log();
            let bytes = include_bytes!("../samplerss.xml");
            let bytes2 = bytes.to_vec();

            let client = Client {};
            let (new_bytes, left, right) = client.parse_valid(bytes2.as_slice());
            debug!("left {}, right {}", left, right);
            debug!("{}", String::from_utf8_lossy(new_bytes.as_slice()));
        }
    }
}
