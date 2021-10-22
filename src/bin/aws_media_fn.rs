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
    let _dest_buck = std::env::var("DEST_BUCK")
        .map_err(|_| SimpleError::new("Dest Bucket not set".to_string()))?;

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
            Ok(ep) => {
                save_to_s3(ep.url.as_str(), "sls11", "ep1.mp3")
                    .await
                    .map_err(|_| SimpleError::new("s3 save failed"))?;
            }
            Err(e) => log::debug!("err {:?}", e),
        }
    }

    Ok(Response {
        request_id: ctx.request_id,
        msg: "".to_string(),
    })
}

use futures::stream::StreamExt;
use sloppy_auth::{aws::s3::Sign, util as u2};
use std::convert::TryInto;
use url::Url;

async fn save_to_s3(url: &str, _bucket: &str, _key: &str) -> Result<(), reqwest::Error> {
    let access_key = std::env::var("AWS_ACCESS_KEY_ID").expect("access key empty");
    let secret_key = std::env::var("AWS_SECRET_ACCESS_KEY").expect("secret key empty");
    let access_token = std::env::var("AWS_SESSION_TOKEN").expect("session token empty");
    log::debug!("access key {}, secret {}", access_key, secret_key,);

    let resp1 = reqwest::get(url).await?;

    let date = chrono::Utc::now();
    let host1 = "sls11.s3.amazonaws.com";
    let key1 = "test1";
    let url1 = format!("http://{}/{}", host1, key1);
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert(
        "x-amz-date".to_string(),
        date.format(u2::LONG_DATETIME).to_string(),
    );
    map.insert(
        "X-Amz-Content-Sha256".to_string(),
        u2::UNSIGNED_PAYLOAD.to_string(),
    );
    map.insert("Host".to_string(), host1.to_owned());
    map.insert("X-Amz-Security-Token".to_string(), access_token.to_owned());

    let s3 = Sign {
        method: "PUT",
        url: Url::parse(&url1).expect("url parse failed"),
        datetime: &date,
        region: "us-east-1",
        access_key: &access_key,
        secret_key: &secret_key,
        headers: map.clone(),
    };

    let signature = s3.sign();
    map.insert("Authorization".to_string(), signature);

    let mut stream1 = resp1.bytes_stream();
    let mut count = 1;
    while let Some(item) = stream1.next().await {
        log::debug!("chunk {},{}", count, item.expect("chunk failed").len());
        count += 1;
    }

    let resp2 = reqwest::Client::new()
        .put(url1)
        .headers((&map).try_into().expect("headers convert failed"))
        .body::<&str>("one".into())
        .send()
        .await?;

    log::debug!("S3 response {:?}", resp2);

    Ok(())
}
