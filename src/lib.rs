pub mod client;
pub mod model;
pub mod parser;

use std::error::Error;
pub type ByteRange = (u32, u32);
pub type ItemsInfo = (Vec<model::Item>, u32);
pub type ItemsResult = Result<ItemsInfo, Box<dyn Error>>;
