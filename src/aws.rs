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
                s: Some(n.to_string()),
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
