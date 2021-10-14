use dianliao_cloud::{aws::Dynamo, get_parser, rss::Parser, util};
use lambda_runtime::{handler_fn, Context, Error};
use rusoto_core::Region;
use serde::{Deserialize, Serialize};
use simple_error::SimpleResult;

#[derive(Deserialize)]
struct Request {
    table: String,
    start: usize,
    end: usize,
    region: String,
    rss_url: String,
}

#[derive(Serialize)]
struct Response {
    request_id: String,
    next_start: usize,
    count: usize,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    util::init_log();

    lambda_runtime::run(handler_fn(fetch_save)).await?;
    Ok(())
}

async fn fetch_save(
    Request {
        table,
        start,
        end,
        region,
        rss_url,
    }: Request,
    ctx: Context,
) -> SimpleResult<Response> {
    let rs_region = region.parse::<Region>().unwrap_or(Region::UsEast1);
    let parser = get_parser();
    let (eps, next_start) = parser
        .parse_rss(&rss_url, (start, end))
        .map_err(util::to_simple)?;
    let dynamo = Dynamo::new(rs_region);
    let count = eps.len();
    dynamo.save_eps(&table, eps).map_err(util::to_simple)?;

    Ok(Response {
        request_id: ctx.request_id,
        next_start,
        count,
    })
}
