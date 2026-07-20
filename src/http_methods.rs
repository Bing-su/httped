use bytes::Bytes;
use salvo::oapi::ToSchema;
use salvo::prelude::*;
use salvo::trailing_slash::remove_slash;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::collections::BTreeMap;

use crate::helper::encode_string;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
struct ResponseGet {
    queries: BTreeMap<String, String>,
    headers: BTreeMap<String, String>,
    origin: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
struct ResponsePost {
    queries: BTreeMap<String, String>,
    headers: BTreeMap<String, String>,
    origin: String,
    url: String,
    data: String,
    files: BTreeMap<String, String>,
    form: BTreeMap<String, String>,
    json: Option<Value>,
}

#[endpoint(tags("Http methods"), status_codes(200, 400, 500))]
async fn get(req: &mut Request) -> Json<ResponseGet> {
    let queries: BTreeMap<String, String> = req.parse_queries().unwrap_or_default();
    let headers: BTreeMap<String, String> = req.parse_headers().unwrap_or_default();
    let url: String = req.uri().to_string();
    let origin: String = req
        .header("X-Forwarded-For")
        .unwrap_or_else(|| req.remote_addr().to_string());
    let response = ResponseGet {
        queries,
        headers,
        origin,
        url,
    };
    Json(response)
}

async fn _common(req: &mut Request) -> Result<Json<ResponsePost>, StatusError> {
    let queries: BTreeMap<String, String> = req.parse_queries().unwrap_or_default();
    let headers: BTreeMap<String, String> = req.parse_headers().unwrap_or_default();
    let url: String = req.uri().to_string();
    let origin: String = req
        .header("X-Forwarded-For")
        .unwrap_or_else(|| req.remote_addr().to_string());

    let payload = req
        .payload_with_max_size(1024 * 1024)
        .await
        .map_err(|_| StatusError::bad_request().brief("Payload size exceeds the limit of 1MB"))?;
    let content_type = headers.get("content-type");
    let data = encode_string(payload, content_type.map(|s| s.as_str()));

    let form: BTreeMap<String, String> = req.parse_form().await.unwrap_or_default();
    let json: Option<Value> = req.parse_json().await.unwrap_or_default();

    let file_parts = req.all_files().await;
    let mut file_vec: Vec<(String, String)> = Vec::with_capacity(file_parts.len());
    for file in file_parts {
        let file_name = file.name().unwrap_or_default();
        let path = file.path();
        let content: Bytes = tokio::fs::read(path).await.unwrap_or_default().into();
        let content_type = file.content_type().map(|s| s.to_string());
        let encoded_content = encode_string(&content, content_type.as_deref());
        file_vec.push((file_name.to_string(), encoded_content));
    }
    let files: BTreeMap<String, String> = BTreeMap::from_iter(file_vec);

    let response = ResponsePost {
        queries,
        headers,
        origin,
        url,
        data,
        files,
        form,
        json,
    };
    Ok(Json(response))
}

#[endpoint(tags("Http methods"), status_codes(200, 400, 500))]
pub async fn post(req: &mut Request) -> Result<Json<ResponsePost>, StatusError> {
    _common(req).await
}

#[endpoint(tags("Http methods"), status_codes(200, 400, 500))]
pub async fn delete(req: &mut Request) -> Result<Json<ResponsePost>, StatusError> {
    _common(req).await
}

#[endpoint(tags("Http methods"), status_codes(200, 400, 500))]
pub async fn put(req: &mut Request) -> Result<Json<ResponsePost>, StatusError> {
    _common(req).await
}

#[endpoint(tags("Http methods"), status_codes(200, 400, 500))]
pub async fn patch(req: &mut Request) -> Result<Json<ResponsePost>, StatusError> {
    _common(req).await
}

#[endpoint(tags("Http methods"), status_codes(200, 400, 500))]
pub async fn head(req: &mut Request) -> Result<Json<ResponsePost>, StatusError> {
    _common(req).await
}

#[endpoint(tags("Http methods"), status_codes(200, 400, 500))]
pub async fn options(req: &mut Request) -> Result<Json<ResponsePost>, StatusError> {
    _common(req).await
}

#[endpoint(tags("Http methods"), status_codes(200, 400, 500))]
pub async fn query(req: &mut Request) -> Result<Json<ResponsePost>, StatusError> {
    _common(req).await
}

pub fn http_methods_router() -> Router {
    Router::with_hoop(remove_slash())
        .push(Router::with_path("get").get(get))
        .push(Router::with_path("post").post(post))
        .push(Router::with_path("delete").delete(delete))
        .push(Router::with_path("put").put(put))
        .push(Router::with_path("patch").patch(patch))
        .push(Router::with_path("head").head(head))
        .push(Router::with_path("options").options(options))
}
