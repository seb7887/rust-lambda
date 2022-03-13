use lambda_http::{http::StatusCode, IntoResponse, Request, RequestExt, Response, service_fn};
use serde::{Deserialize};
use serde_json::json;
use tracing::{info, warn};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::Client;
use uuid::Uuid;

type E = Box<dyn std::error::Error + Sync + Send + 'static>;

#[derive(Deserialize, Debug)]
struct RequestBody {
    first_name: String,
    last_name: String,
}

#[tokio::main]
async fn main() -> Result<(), E> {
    lambda_http::run(service_fn(handler)).await?;
    Ok(())
}

async fn handler(event: Request) -> Result<impl IntoResponse, E> {
    let body: RequestBody = match event.payload() {
        Ok(Some(body)) => body,
        Ok(None) => {
            warn!("Missing request body");
            return Ok(response(
                StatusCode::BAD_REQUEST,
                json!({ "message": "Missing request body"}).to_string()
            ));
        }
        Err(err) => {
            warn!("Failed to parse request body: {}", err);
            return Ok(response(
                StatusCode::BAD_REQUEST,
                json!({"message": "Failed to parse body"}).to_string()
            ));
        }
    };
    info!("Body: {:?}", body);
    let uuid = Uuid::new_v4().to_string();
    let region_provider = RegionProviderChain::default_provider().or_else("us-east-1");
    let config = aws_config::from_env().region(region_provider).load().await;
    let client = Client::new(&config);
    let ddb = client
        .put_item()
        .table_name("Contacts_SS")
        .item("id", AttributeValue::S(String::from(&uuid)))
        .item("firstName", AttributeValue::S(String::from(body.first_name)))
        .item("lastName", AttributeValue::S(String::from(body.last_name)));
    ddb.send().await?;

    Ok(response(
        StatusCode::OK,
        json!({"message": format!("Hello {} {}!", body.first_name, body.last_name)}).to_string()
    ))
}

fn response(status_code: StatusCode, body: String) -> Response<String> {
    Response::builder()
        .status(status_code)
        .header("Content-Type", "application/json")
        .body(body)
        .unwrap()
}

#[cfg(test)]
mod test {
    use super::*;
    use lambda_http::{http, Body};

    #[tokio::test]
    async fn test_handler() {
        let request = http::Request::builder()
            .uri("https://example.com")
            .header("Content-Type", "application/json")
            .body(Body::from(json!({"first_name": "john", "last_name": "doe"}).to_string()))
            .expect("failed to build request");
        let expected = json!({
            "message": "Hello john doe!"
        }).into_response();
        let response = handler(request)
            .await
            .expect("expected Ok(...) value")
            .into_response();
        assert_eq!(response.body(), expected.body())
    }
}