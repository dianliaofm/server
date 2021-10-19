use dianliao_cloud::util;
use lambda_runtime::{handler_fn, Context, Error};
//use rusoto_core::Region;
use serde::{Deserialize, Serialize};
use simple_error::{SimpleResult, SimpleError};
use std::collections::HashMap;

#[derive(Deserialize)]
struct Request {
    #[serde(rename = "Records")]
    records: Vec<Record>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Record {
    aws_region: String,
    dynamodb: DB,
    event_name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct DB {
    #[serde(rename = "ApproximateCreationDateTime")]
    time: f32,
    keys: Attrs,
    new_image: Attrs,
}

type Attrs = HashMap<String, Attr>;

#[derive(Deserialize, Debug)]
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

    let dest_buck = std::env::var("DEST_BUCK").map_err(|_| SimpleError::new("Dest Bucket not set".to_string()))?;
    log::debug!("save to {}", dest_buck);

    let mut msg = String::from("");

    for r in req.records {
        log::debug!("{:?}", r);
    }
    Ok(Response {
        request_id: ctx.request_id,
        msg,
    })
}
