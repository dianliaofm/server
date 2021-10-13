use crate::entity::Episode;
use rusoto_core::Region;
use rusoto_dynamodb::{
    AttributeValue, BatchWriteItemInput, DynamoDb, DynamoDbClient, PutRequest, WriteRequest,
};
use std::collections::HashMap;
use std::error::Error;

pub struct Dynamo {
    db_client: DynamoDbClient,
}

impl Dynamo {
    pub fn new(region: Region) -> Self {
        Dynamo {
            db_client: DynamoDbClient::new(region),
        }
    }
    pub fn save_eps(&self, table: &str, eps: Vec<Episode>) -> Result<(), Box<dyn Error>> {
        let mut items = HashMap::new();
        items.insert(
            table.to_string(),
            eps.iter()
                .map(|e: &Episode| e.clone().into())
                .map(|p: PutRequest| WriteRequest {
                    put_request: Some(p),
                    ..Default::default()
                })
                .collect::<Vec<WriteRequest>>(),
        );
        let input = BatchWriteItemInput {
            request_items: items,
            ..Default::default()
        };
        self.db_client.batch_write_item(input).sync()?;
        Ok(())
    }
}

struct PutBuilder {
    map: HashMap<String, AttributeValue>,
}

impl PutBuilder {
    fn new() -> Self {
        PutBuilder {
            map: HashMap::new(),
        }
    }
    fn set_s(mut self, k: &str, s: &str) -> Self {
        self.map.insert(
            k.to_string(),
            AttributeValue {
                s: Some(s.to_string()),
                ..Default::default()
            },
        );
        self
    }

    fn set_n(mut self, k: &str, n: u64) -> Self {
        self.map.insert(
            k.to_string(),
            AttributeValue {
                n: Some(n.to_string()),
                ..Default::default()
            },
        );
        self
    }

    fn build(self) -> PutRequest {
        PutRequest { item: self.map }
    }
}

impl From<Episode> for PutRequest {
    fn from(ep: Episode) -> Self {
        PutBuilder::new()
            .set_s("title", &ep.title)
            .set_s("subtitle", &ep.subtitle)
            .set_s("description", &ep.description)
            .set_s("date", &ep.date)
            .set_n("timestamp", ep.timestamp)
            .set_s("url", &ep.url)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::init_log;

    #[test]
    fn save_episodes() {
        init_log();
        let eps = (0..5)
            .map(|x| Episode {
                title: format!("title test {}", x),
                subtitle: format!("sub test {}", x),
                description: format!("desc test {}", x),
                date: format!("date test {}", x),
                timestamp: 2000 + x as u64,
                url: format!("url test {}", x),
            })
            .collect::<Vec<Episode>>();
        let dn = Dynamo::new(Region::UsEast1);
        dn.save_eps("test1", eps).expect("save episode failed");
    }
}
