use salvo::catcher::Catcher;
use salvo::http::header::CONTENT_TYPE;
use salvo::http::{HeaderValue, ResBody};
use salvo::oapi::{Components, Content, EndpointOutRegister, Operation, RefOr, ToSchema};
use salvo::prelude::*;
use salvo::{Error, Scribe};
use serde::Serialize;

const PROBLEM_JSON: &str = "application/problem+json";

#[derive(Debug, Serialize, ToSchema)]
struct ProblemDetails {
    r#type: String,
    title: String,
    status: u16,
    detail: String,
    instance: String,
}

#[derive(Debug)]
pub struct ApiError(Error);

pub type ApiResult<T> = std::result::Result<T, ApiError>;

impl From<StatusError> for ApiError {
    fn from(error: StatusError) -> Self {
        Self(error.into())
    }
}

impl From<Error> for ApiError {
    fn from(error: Error) -> Self {
        Self(error)
    }
}

impl Scribe for ApiError {
    fn render(self, res: &mut Response) {
        self.0.render(res);
    }
}

impl EndpointOutRegister for ApiError {
    fn register(components: &mut Components, operation: &mut Operation) {
        StatusError::register(components, operation);
        let problem_schema = ProblemDetails::to_schema(components);

        for (code, response) in operation.responses.iter_mut() {
            if !code
                .parse::<u16>()
                .is_ok_and(|code| (400..600).contains(&code))
            {
                continue;
            }
            let RefOr::Type(response) = response else {
                continue;
            };
            response.contents.clear();
            response.contents.insert(
                PROBLEM_JSON.to_owned(),
                Content::new(problem_schema.clone()),
            );
        }
    }
}

#[handler]
async fn render_problem(req: &mut Request, res: &mut Response) {
    let status = res.status_code.unwrap_or(StatusCode::NOT_FOUND);
    let (title, detail) = match &res.body {
        ResBody::Error(error) => (error.name.clone(), error.brief.clone()),
        _ => {
            let title = status.canonical_reason().unwrap_or("HTTP Error").to_owned();
            (title.clone(), title)
        }
    };

    res.render(Json(ProblemDetails {
        r#type: "about:blank".to_owned(),
        title,
        status: status.as_u16(),
        detail,
        instance: req
            .uri()
            .path_and_query()
            .map_or(req.uri().path(), |value| value.as_str())
            .to_owned(),
    }));
    res.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static(PROBLEM_JSON));
}

pub(crate) fn catcher() -> Catcher {
    Catcher::new(render_problem)
}

#[cfg(test)]
mod tests {
    use salvo::http::StatusCode;
    use salvo::prelude::{Router, Service};
    use salvo::test::{ResponseExt, TestClient};
    use serde_json::{Value, json};

    use super::{CONTENT_TYPE, PROBLEM_JSON, catcher};
    use crate::cli::{build_router, build_service};

    #[tokio::test]
    async fn rfc9457_contract() {
        let service = build_service();

        let mut response = TestClient::get("http://localhost/bearer?source=test")
            .send(&service)
            .await;
        assert_eq!(response.status_code, Some(StatusCode::UNAUTHORIZED));
        assert_eq!(response.headers()[CONTENT_TYPE], PROBLEM_JSON);
        assert_eq!(
            response.take_json::<Value>().await.unwrap(),
            json!({
                "type": "about:blank",
                "title": "Unauthorized",
                "status": 401,
                "detail": "Missing authorization header",
                "instance": "/bearer?source=test"
            })
        );

        let mut response = TestClient::get("http://localhost/missing")
            .send(&service)
            .await;
        assert_eq!(response.status_code, Some(StatusCode::NOT_FOUND));
        assert_eq!(response.headers()[CONTENT_TYPE], PROBLEM_JSON);
        assert_eq!(response.take_json::<Value>().await.unwrap()["status"], 404);

        let mut response = TestClient::get("http://localhost/status/get/404")
            .send(&service)
            .await;
        assert_eq!(response.status_code, Some(StatusCode::NOT_FOUND));
        assert_eq!(response.headers()[CONTENT_TYPE], PROBLEM_JSON);
        assert_eq!(response.take_json::<Value>().await.unwrap()["status"], 404);

        let mut response = TestClient::get("http://localhost/status/get/200")
            .send(&service)
            .await;
        assert_eq!(response.status_code, Some(StatusCode::OK));
        assert_eq!(response.take_string().await.unwrap(), "");

        let mut response = TestClient::get("http://localhost/openapi.json")
            .send(&service)
            .await;
        let doc = response.take_json::<Value>().await.unwrap();
        let bearer = &doc["paths"]["/bearer"]["get"]["responses"]["401"]["content"];
        assert!(bearer.get(PROBLEM_JSON).is_some());
        assert_eq!(bearer.as_object().unwrap().len(), 1);
        for code in ["400", "500"] {
            let content = &doc["paths"]["/get"]["get"]["responses"][code]["content"];
            assert!(content.get(PROBLEM_JSON).is_some());
            assert_eq!(content.as_object().unwrap().len(), 1);
        }
        let status = &doc["paths"]["/status/get/{code}"]["get"]["responses"]["400"]["content"];
        assert!(status.get(PROBLEM_JSON).is_some());
        assert_eq!(status.as_object().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn problem_details_preserve_mount_path() {
        let service =
            Service::new(Router::with_path("api").push(build_router())).catcher(catcher());

        let mut response = TestClient::get("http://localhost/api/bearer?source=test")
            .send(&service)
            .await;

        assert_eq!(
            response.take_json::<Value>().await.unwrap()["instance"],
            "/api/bearer?source=test"
        );

        let mut response = TestClient::get("http://localhost/api/status/get/404")
            .send(&service)
            .await;
        assert_eq!(response.status_code, Some(StatusCode::NOT_FOUND));
        assert_eq!(
            response.take_json::<Value>().await.unwrap()["instance"],
            "/api/status/get/404"
        );

        let mut response = TestClient::get("http://localhost/api/status/get/9999")
            .send(&service)
            .await;
        assert_eq!(response.status_code, Some(StatusCode::BAD_REQUEST));
        assert_eq!(
            response.take_json::<Value>().await.unwrap()["instance"],
            "/api/status/get/9999"
        );
    }
}
