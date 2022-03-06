use lambda_runtime::{Context, handler_fn, error::HandlerError};
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use std::error::Error;

#[derive(Deserialize)]
struct Request {
    first_name: String,
    last_name: String,
}

#[derive(Serialize)]
struct Response {
    msg: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    SimpleLogger::new().with_level(LevelFilter::Info).init().unwrap();

    let func = handler_fn(handler);
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn handler(event: Request, _: Context) -> Result<Response, HandlerError> {
    let resp = Response {
        msg: format!("Hello {}!", event.first_name),
    };

    Ok(resp)
}
