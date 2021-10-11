use std::io::BufRead;

pub type ItemStart = usize;
pub type ItemEnd = usize;
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

            let mut left: Option<usize> = None;
            let mut right = 0usize;

            let mut event_list: Vec<Event> = Vec::new();

            loop {
                match reader.read_event(&mut buf) {
                    Ok(Event::Eof) => break,
                    Ok(Event::Start(e)) if e.name() == ITEM => {
                        if let None = left {
                            left = Some(reader.buffer_position());
                        }
                        event_list.push(Event::Start(e.into_owned()));
                    }
                    Ok(Event::End(e)) if e.name() == ITEM => {
                        if !event_list.is_empty() {
                            right = reader.buffer_position();
                            for ev in &event_list {
                                assert!(writer.write_event(ev).is_ok())
                            }
                            assert!(writer.write_event(Event::End(e.into_owned())).is_ok());
                            event_list.clear();
                        }
                    }
                    Ok(e) => {
                        if !event_list.is_empty() {
                            event_list.push(e.into_owned());
                        }
                    }
                    Err(e) => log::debug!("Error at {}: {:?}", reader.buffer_position(), e),
                }
                buf.clear();
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
        use std::include_bytes;

        #[test]
        fn xml_valid() {
            init_log();
            let bytes = include_bytes!("../samplerss.xml");
            let bytes2 = bytes.to_vec();

            let client = Client {};
            let (new_bytes, left, right) = client.parse_valid(bytes2.as_slice());
            let item_tag1 = &bytes2[(left - 6)..left];
            let item_tag2 = &bytes2[(right - 7)..right];
            assert_eq!(b"<item>", item_tag1);
            assert_eq!(b"</item>", item_tag2);

            assert_eq!(b"<item>", &new_bytes[0..6]);
            let new_len = new_bytes.len();
            assert_eq!(b"</item>", &new_bytes[(new_len - 7)..new_len]);
        }
    }
}
