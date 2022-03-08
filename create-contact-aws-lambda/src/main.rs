use lambda_http::{http::StatusCode, service_fn, Error, IntoResponse, Request, Response};
use serde::{Deserialize};
use serde_json::json;
use tracing::{info, warn};

#[derive(Deserialize, Debug)]
struct RequestBody {
    first_name: String,
    last_name: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_http::run(service_fn(handler)).await?;
    Ok(())
}

async fn handler(event: Request) -> Result<impl IntoResponse, Error> {
    let body: RequestBody = match event.payload() {
        Ok(Some(body)) => body,
        Ok(None) => {
            warn!("Missing request body")
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
    }
    info!("Body: {:?}", body);

    Ok(response(
        StatusCode::OK,
        json!({"message": "Hello World!"}).to_string()
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

    #[tokio::test]
    async fn test_handler() {
        let request = lambda_http::http:Request::builder()
            .body(json!({"first_name": "John", "last_name": "Doe"}).to_string());
        let expected = json!({
            "message": "Hello World!"
        }).into_response();
        let response = handler(request)
            .await
            .expect("expected Ok(...) value")
            .into_response();
        assert_eq!(response.body(), expected.body())
    }
}