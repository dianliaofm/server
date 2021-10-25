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
                save_to_s3(ep.url.as_str(), "sls11", "ep1.mp3")?;
            }
            Err(e) => log::debug!("err {:?}", e),
        }
    }

    Ok(Response {
        request_id: ctx.request_id,
        msg: "".to_string(),
    })
}

use sloppy_auth::{aws::s3, chunk, util as u2};
use url::Url;

fn save_to_s3(url: &str, bucket: &str, key: &str) -> SimpleResult<()> {
    let access_key = std::env::var("AWS_ACCESS_KEY_ID").expect("access key empty");
    let secret_key = std::env::var("AWS_SECRET_ACCESS_KEY").expect("secret key empty");
    let access_token = std::env::var("AWS_SESSION_TOKEN").expect("session token empty");

    let response1 = ureq::get(&url)
        .call()
        .map_err(|e| SimpleError::new(e.to_string()))?;
    let content_len: String = response1
        .header("Content-Length")
        .expect("content length empty")
        .to_string();
    let rd1 = response1.into_reader();

    let host = "s3.amazonaws.com";
    let date = chrono::Utc::now();
    let host1 = format!("{}.{}", bucket, host);
    let full_url = format!("http://{}/{}", host1, key);
    let mut headers: HashMap<String, String> = HashMap::new();
    headers.insert("Host".to_string(), host1.to_owned());
    headers.insert(
        "X-Amz-Content-Sha256".to_string(),
        s3::STREAM_PAYLOAD.to_string(),
    );

    headers.insert("Content-Encoding".to_string(), "aws-chunked".to_string());
    headers.insert("x-amz-decoded-content-length".to_string(), content_len);
    headers.insert("Transfer-Encoding".to_string(), "chunked".to_string());
    headers.insert(
        "x-amz-date".to_string(),
        date.format(u2::LONG_DATETIME).to_string(),
    );
    headers.insert("X-Amz-Security-Token".to_string(), access_token.to_owned());

    let signer = s3::Sign {
        method: "PUT",
        url: Url::parse(&full_url).expect("url parse failed"),
        datetime: &date,
        region: "us-east-1",
        access_key: &access_key,
        secret_key: &secret_key,
        headers: headers.clone(),
        transfer_mode: s3::Transfer::Multiple,
    };

    headers.insert("Authorization".to_string(), signer.sign());
    let holder = s3::api::Holder::new(6 * 1024 * 1024, rd1, signer);
    let chunk = chunk::Chunk::new(holder);
    let mut request2 = ureq::put(&full_url);
    for (k, v) in headers {
        request2 = request2.set(&k, &v);
    }

    request2
        .send(chunk)
        .map_err(|_| SimpleError::new("save to s3 failed"))?;

    Ok(())
}
