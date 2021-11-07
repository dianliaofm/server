use crate::entity::Episode;
use rusoto_core::Region;
use rusoto_dynamodb::{BatchWriteItemInput, DynamoDb, DynamoDbClient, PutRequest, WriteRequest};
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
    pub async fn save_eps(&self, table: &str, eps: Vec<Episode>) -> Result<(), Box<dyn Error>> {
        let mut items = HashMap::new();
        items.insert(
            table.to_string(),
            eps.iter()
                .map(|e: &Episode| PutRequest {
                    item: serde_dynamodb::to_hashmap(e).expect("putrequest item failed"),
                })
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
        self.db_client.batch_write_item(input).await?;
        Ok(())
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
                image: format!("http://testimage{}.jpg", x),
                date_key: format!("20090{}", x),
            })
            .collect::<Vec<Episode>>();
        let dn = Dynamo::new(Region::UsEast1);
        dn.save_eps("test1", eps).expect("save episode failed");
    }
}
