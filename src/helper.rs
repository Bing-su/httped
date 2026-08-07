use std::collections::BTreeMap;

use bytes::Bytes;
use data_encoding::BASE64;
use rust_embed::RustEmbed;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::rfc9457::ApiResult;

const DEFAULT_PAYLOAD_MAX_SIZE: usize = 1024 * 1024; // 1 MB

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub(crate) struct GetResponse {
    queries: BTreeMap<String, String>,
    headers: BTreeMap<String, String>,
    origin: String,
    url: String,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub(crate) struct PostResponse {
    queries: BTreeMap<String, String>,
    headers: BTreeMap<String, String>,
    origin: String,
    url: String,
    data: String,
    files: BTreeMap<String, String>,
    form: BTreeMap<String, String>,
    json: Option<Value>,
}

pub(crate) fn encode_string(input: &Bytes, content_type: Option<&str>) -> String {
    match String::from_utf8(input.to_vec()) {
        Ok(s) => s,
        Err(_) => format!(
            "data:{};base64,{}",
            content_type.unwrap_or("application/octet-stream"),
            BASE64.encode(input)
        ),
    }
}

#[derive(RustEmbed)]
#[folder = "asset"]
pub struct Asset;

pub(crate) fn get(req: &mut Request) -> Json<GetResponse> {
    let queries: BTreeMap<String, String> = req.parse_queries().unwrap_or_default();
    let headers: BTreeMap<String, String> = req.parse_headers().unwrap_or_default();
    let url: String = req.uri().to_string();
    let origin: String = req
        .header("X-Forwarded-For")
        .unwrap_or_else(|| req.remote_addr().to_string());
    let response = GetResponse {
        queries,
        headers,
        origin,
        url,
    };
    Json(response)
}

pub(crate) async fn post(req: &mut Request) -> ApiResult<Json<PostResponse>> {
    let queries: BTreeMap<String, String> = req.parse_queries().unwrap_or_default();
    let headers: BTreeMap<String, String> = req.parse_headers().unwrap_or_default();
    let url: String = req.uri().to_string();
    let origin: String = req
        .header("X-Forwarded-For")
        .unwrap_or_else(|| req.remote_addr().to_string());

    let payload = req
        .payload_with_max_size(DEFAULT_PAYLOAD_MAX_SIZE)
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

    let response = PostResponse {
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
