use std::convert::Infallible;
use std::time::Duration;

use async_stream::stream;
use salvo::oapi::extract::*;
use salvo::prelude::*;
use salvo::sse::{SseEvent, SseKeepAlive};
use salvo::trailing_slash::remove_slash;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const DEFAULT_INTERVAL: f64 = 0.1;
const DEFAULT_TIMEOUT: u64 = 30;

#[derive(Serialize, Deserialize, ToParameters, ToSchema, Debug)]
struct CounterParams {
    #[salvo(parameter(parameter_in = "path", default = 3))]
    count: u32,
}

#[derive(Serialize, Deserialize, ToParameters, ToSchema, Debug)]
struct IntervalParams {
    #[salvo(parameter(parameter_in = "query", default = 0.1, minimum = 0.0))]
    interval: Option<f64>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
struct TextRequest {
    #[salvo(schema(default = "Hello\nWorld"))]
    text: String,
}

#[endpoint(tags("SSE"), status_codes(200, 400, 500))]
async fn counter_text(counter: CounterParams, interval: IntervalParams, res: &mut Response) {
    let count = counter.count;
    let itv = interval.interval.unwrap_or(DEFAULT_INTERVAL);
    let event_stream = stream! {
        for i in 1..=count {
            if itv > 0.0 {
                tokio::time::sleep(Duration::from_secs_f64(itv)).await;
            }
            yield Ok::<_, Infallible>(SseEvent::default().text(i.to_string()));
        };
    };
    SseKeepAlive::new(event_stream)
        .max_interval(Duration::from_secs(5))
        .comment(format!("/sse/counter/text/{}", count))
        .stream(res);
}

#[endpoint(tags("SSE"), status_codes(200, 400, 500))]
async fn counter_json(counter: CounterParams, interval: IntervalParams, res: &mut Response) {
    let count = counter.count;
    let itv = interval.interval.unwrap_or(DEFAULT_INTERVAL);
    let event_stream = stream! {
        for i in 1..=count {
            if itv > 0.0 {
                tokio::time::sleep(Duration::from_secs_f64(itv)).await;
            }
            yield SseEvent::default().json(json!({"count": i}));
        };
    };
    SseKeepAlive::new(event_stream)
        .max_interval(Duration::from_secs(5))
        .comment(format!("/sse/counter/json/{}", count))
        .stream(res);
}

#[endpoint(tags("SSE"), status_codes(200, 400, 500))]
async fn split_text(
    body: JsonBody<TextRequest>,
    interval: IntervalParams,
    res: &mut Response,
) -> Result<(), StatusError> {
    let itv = interval.interval.unwrap_or(DEFAULT_INTERVAL);
    let event_stream = stream! {
        for line in body.text.lines() {
            if itv > 0.0 {
                tokio::time::sleep(Duration::from_secs_f64(itv)).await;
            }
            yield Ok::<_, Infallible>(SseEvent::default().text(line.to_string()));
        };
    };
    SseKeepAlive::new(event_stream)
        .max_interval(Duration::from_secs(5))
        .comment("/sse/text")
        .stream(res);
    Ok(())
}

#[endpoint(tags("SSE"), status_codes(200, 400, 500))]
async fn split_json(
    body: JsonBody<Value>,
    interval: IntervalParams,
    res: &mut Response,
) -> Result<(), StatusError> {
    let itv = interval.interval.unwrap_or(DEFAULT_INTERVAL);
    let json_text = serde_json::to_string_pretty(&body.into_inner())
        .map_err(|e| StatusError::internal_server_error().brief(e.to_string()))?;
    let event_stream = stream! {
        for line in json_text.lines() {
            if itv > 0.0 {
                tokio::time::sleep(Duration::from_secs_f64(itv)).await;
            }
            yield Ok::<_, Infallible>(SseEvent::default().text(line.to_string()));
        };
    };
    SseKeepAlive::new(event_stream)
        .max_interval(Duration::from_secs(5))
        .comment("/sse/json")
        .stream(res);
    Ok(())
}

pub fn sse_router() -> Router {
    Router::with_hoop(remove_slash())
        .hoop(Timeout::new(Duration::from_secs(DEFAULT_TIMEOUT)))
        .push(Router::with_path("sse/counter/text/{count}").get(counter_text))
        .push(Router::with_path("sse/counter/json/{count}").get(counter_json))
        .push(Router::with_path("sse/text").post(split_text))
        .push(Router::with_path("sse/json").post(split_json))
}

#[cfg(test)]
mod tests {
    use super::*;
    use salvo::http::header::CONTENT_TYPE;
    use salvo::test::{ResponseExt, TestClient};

    #[tokio::test]
    async fn counter_text_stream() {
        let service = Service::new(sse_router());
        let mut response = TestClient::get("http://localhost/sse/counter/text/3?interval=0")
            .send(&service)
            .await;

        assert_eq!(response.status_code, Some(StatusCode::OK));
        assert_eq!(response.headers()[CONTENT_TYPE], "text/event-stream");
        assert_eq!(
            response.take_string().await.unwrap(),
            "data:1\n\ndata:2\n\ndata:3\n\n"
        );
    }

    #[tokio::test]
    async fn counter_json_stream() {
        let service = Service::new(sse_router());
        let mut response = TestClient::get("http://localhost/sse/counter/json/3?interval=0")
            .send(&service)
            .await;

        assert_eq!(response.status_code, Some(StatusCode::OK));
        assert_eq!(response.headers()[CONTENT_TYPE], "text/event-stream");
        assert_eq!(
            response.take_string().await.unwrap(),
            "data:{\"count\":1}\n\ndata:{\"count\":2}\n\ndata:{\"count\":3}\n\n"
        );
    }

    #[tokio::test]
    async fn split_text_stream() {
        let service = Service::new(sse_router());
        let mut response = TestClient::post("http://localhost/sse/text")
            .json(&TextRequest {
                text: "Hello\nWorld\nThis\r\nIs\nA\nTest".to_string(),
            })
            .send(&service)
            .await;

        assert_eq!(response.status_code, Some(StatusCode::OK));
        assert_eq!(response.headers()[CONTENT_TYPE], "text/event-stream");
        assert_eq!(
            response.take_string().await.unwrap(),
            "data:Hello\n\ndata:World\n\ndata:This\n\ndata:Is\n\ndata:A\n\ndata:Test\n\n"
        );
    }

    #[tokio::test]
    async fn split_json_stream() {
        let service = Service::new(sse_router());
        let mut response = TestClient::post("http://localhost/sse/json")
            .json(&json!({
                "foo": "bar",
                "baz": [1, 2, 3],
                "nested": {
                    "a": true,
                    "b": null,
                    "c": 3.14
                }
            }))
            .send(&service)
            .await;

        assert_eq!(response.status_code, Some(StatusCode::OK));
        assert_eq!(response.headers()[CONTENT_TYPE], "text/event-stream");
        assert_eq!(
            response.take_string().await.unwrap(),
            "data:{\n\ndata:  \"baz\": [\n\ndata:    1,\n\ndata:    2,\n\ndata:    3\n\ndata:  ],\n\ndata:  \"foo\": \"bar\",\n\ndata:  \"nested\": {\n\ndata:    \"a\": true,\n\ndata:    \"b\": null,\n\ndata:    \"c\": 3.14\n\ndata:  }\n\ndata:}\n\n"
        );
    }
}
