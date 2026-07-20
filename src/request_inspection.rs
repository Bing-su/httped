use salvo::prelude::*;
use salvo::trailing_slash::remove_slash;
use serde::Serialize;

use std::collections::BTreeMap;

#[derive(Serialize, Debug, ToSchema)]
struct HeadersResponse {
    headers: BTreeMap<String, String>,
}

#[derive(Serialize, Debug, ToSchema)]
struct IPResponse {
    ip: String,
}

#[derive(Serialize, Debug, ToSchema)]
struct UserAgentResponse {
    user_agent: String,
}

#[endpoint(tags("Request inspection"), status_codes(200))]
async fn headers_(req: &mut Request) -> Json<HeadersResponse> {
    let headers: BTreeMap<String, String> = req.parse_headers().unwrap_or_default();
    let response = HeadersResponse { headers };
    Json(response)
}

#[endpoint(tags("Request inspection"))]
async fn ip_(req: &mut Request) -> Json<IPResponse> {
    let ip = req
        .header("X-Forwarded-For")
        .unwrap_or_else(|| req.remote_addr().to_string());
    let response = IPResponse { ip };
    Json(response)
}

#[endpoint(tags("Request inspection"))]
async fn user_agent_(req: &mut Request) -> Json<UserAgentResponse> {
    let user_agent = req.header("User-Agent").unwrap_or_default();
    let response = UserAgentResponse { user_agent };
    Json(response)
}

pub fn request_inspection_router() -> Router {
    Router::with_hoop(remove_slash())
        .push(Router::with_path("headers").get(headers_))
        .push(Router::with_path("ip").get(ip_))
        .push(Router::with_path("user-agent").get(user_agent_))
}
