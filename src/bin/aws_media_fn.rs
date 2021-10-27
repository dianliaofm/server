use dianliao_cloud::{entity::Episode, util};
use lambda_runtime::{handler_fn, Context, Error};
use serde::{Deserialize, Serialize};
use simple_error::{SimpleError, SimpleResult};
use std::collections::HashMap;

#[derive(Deserialize)]
struct Request {
    #[serde(rename = "Records")]
    records: Vec<Record>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct Record {
    aws_region: String,
    dynamodb: DB,
    event_name: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
struct DB {
    #[serde(rename = "ApproximateCreationDateTime")]
    time: f32,
    keys: Attrs,
    new_image: Attrs,
}

type Attrs = HashMap<String, Attr>;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "UPPERCASE")]
struct Attr {
    s: Option<String>,
    n: Option<String>,
}

#[derive(Serialize)]
struct Response {
    request_id: String,
    msg: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    util::init_log();

    lambda_runtime::run(handler_fn(fetch_save)).await?;
    Ok(())
}

async fn fetch_save(req: Request, ctx: Context) -> SimpleResult<Response> {
    let dest_buck = std::env::var("DEST_BUCK")
        .map_err(|_| SimpleError::new("Dest Bucket not set".to_string()))?;
    let chunk_kb =
        std::env::var("CHUNK_KB").map_err(|_| SimpleError::new("Chunk kb empty".to_string()))?;
    let ep_prefix =
        std::env::var("EP_PREFIX").map_err(|_| SimpleError::new("Ep prefix empty".to_string()))?;
    let region = std::env::var("AWS_REGION").unwrap_or("us-east-1".to_string());

    let results: Vec<SimpleResult<Episode>> = req
        .records
        .iter()
        .map(|r: &Record| {
            let r1 = r.clone();
            match r1.event_name.as_str() {
                "INSERT" => {
                    let new_img: Attrs = r1.dynamodb.new_image;
                    let attr1: Attr = new_img.get("timestamp").unwrap().clone();
                    let timestamp: String = attr1.n.unwrap();
                    let timestamp = timestamp.parse::<u64>().unwrap();
                    let attr2 = new_img.get("url").unwrap().clone();
                    let url = attr2.s.unwrap();

                    let attr3 = new_img.get("title").unwrap().clone();
                    let title = attr3.s.unwrap();

                    Ok(Episode {
                        timestamp,
                        url,
                        title,
                        ..Default::default()
                    })
                }
                t => Err(SimpleError::new(format!("event name {}", t))),
            }
        })
        .collect();

    let saver = Saver {
        region,
        bucket: dest_buck,
        ep_prefix,
        chunk_kb: chunk_kb.parse::<usize>().unwrap_or(256),
    };

    let mut msg = Vec::<String>::with_capacity(1);
    for x in results {
        match x {
            Ok(ep) => {
                saver.save_to_s3(&ep)?;
                msg.push(ep.title);
            }
            Err(e) => msg.push(e.to_string()),
        }
    }

    Ok(Response {
        request_id: ctx.request_id,
        msg: msg.join("\n"),
    })
}

use sloppy_auth::aws::s3::client::{ChunkExt, Client};

struct Saver {
    region: String,
    bucket: String,
    ep_prefix: String,
    chunk_kb: usize,
}

impl Saver {
    fn save_to_s3(&self, ep: &Episode) -> SimpleResult<()> {
        let client = Client::new(self.region.clone());
        let key = format!("{}{}", self.ep_prefix, ep.timestamp);
        client.save_remote(&ep.url, self.chunk_kb, &self.bucket, &key)?;
        Ok(())
    }
}
