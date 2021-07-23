use curl::easy::{Easy, List};

type ByteRange = (u32, u32);

pub struct Rss {
    rss_url: String,
    range: ByteRange,
}

impl Rss {
    pub fn fetch(&self) -> String {
        let (start, end) = self.range;
        let mut list = List::new();
        let mut data = Vec::new();
        list.append(&format!("Range: bytes={}-{}", start, end))
            .unwrap();
        let mut handle = Easy::new();
        handle.url(&self.rss_url).unwrap();
        handle.http_headers(list).unwrap();
        {
            let mut transfer = handle.transfer();
            transfer
                .write_function(|s| {
                    data.extend_from_slice(s);
                    Ok(s.len())
                })
                .unwrap();
            transfer.perform().unwrap();
        }
        String::from_utf8(data).unwrap()
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
            range: (0u32, 2800u32),
        };
        let s = rss.fetch();
        println!("{}", s);
    }
}
