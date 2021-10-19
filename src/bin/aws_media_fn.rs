use dianliao_cloud::{entity::Episode, util};
use lambda_runtime::{handler_fn, Context, Error};
//use rusoto_core::Region;
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
    log::debug!("save to {}", dest_buck);

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
                    Ok(Episode {
                        timestamp,
                        url,
                        ..Default::default()
                    })
                }
                t => Err(SimpleError::new(format!("event name {}", t))),
            }
        })
        .collect();

    for x in results {
        match x {
            Ok(ep) => log::debug!("ep {:?}", ep),
            Err(e) => log::debug!("err {:?}", e),
        }
    }

    Ok(Response {
        request_id: ctx.request_id,
        msg: "".to_string(),
    })
}
