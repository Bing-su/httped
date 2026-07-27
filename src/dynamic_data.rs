use std::convert::Infallible;

use async_stream::stream;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use rand::prelude::*;
use rand::rngs::ChaCha20Rng;
use salvo::Error;
use salvo::http::header::{CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE};
use salvo::prelude::*;
use salvo::trailing_slash::remove_slash;
use serde::{Deserialize, Serialize};

use crate::helper::{PostResponse, post};

const DEFAULT_CHUNK_SIZE: usize = 8 * 1024;
const MAX_BYTES: u32 = 1000000; // 1 MB

#[derive(Serialize, Deserialize, ToParameters, ToSchema, Debug)]
struct BytesSimpleRequest {
    /// Number of bytes.
    #[salvo(parameter(parameter_in = "path", minimum = 0, maximum = 1000000, default = 1024))]
    n: u32,
    /// Random seed.
    #[salvo(parameter(parameter_in = "query", minimum = 0))]
    seed: Option<u64>,
    /// Download filename.
    #[salvo(parameter(parameter_in = "query", min_length = 1, max_length = 128))]
    filename: Option<String>,
}

#[derive(Serialize, Deserialize, ToParameters, ToSchema, Debug)]
struct BytesStreamRequest {
    /// Number of bytes.
    #[salvo(parameter(parameter_in = "path", minimum = 0, maximum = 1000000, default = 1024))]
    n: u32,
    /// Random seed.
    #[salvo(parameter(parameter_in = "query", minimum = 0))]
    seed: Option<u64>,
    /// Chunk size in bytes.
    #[salvo(parameter(parameter_in = "query", minimum = 1, default = 8192))]
    chunk_size: Option<usize>,
    /// Download filename.
    #[salvo(parameter(parameter_in = "query", min_length = 1, max_length = 128))]
    filename: Option<String>,
}

#[derive(Serialize, ToSchema, Debug)]
struct UuidResponse {
    uuid: String,
}

#[derive(Serialize, ToSchema, Debug)]
struct UlidResponse {
    ulid: String,
}

#[derive(Serialize, Deserialize, ToParameters, ToSchema, Debug)]
struct DelayRequest {
    /// Delay in seconds.
    #[salvo(parameter(parameter_in = "path", minimum = 0, maximum = 10, default = 1))]
    seconds: u64,
}

fn disposition_header(filename: Option<String>) -> String {
    match filename {
        Some(name) => {
            let encoded = utf8_percent_encode(&name, NON_ALPHANUMERIC).to_string();
            format!(
                "attachment; filename={}; filename*=utf-8''{}",
                encoded, encoded
            )
        }
        None => "attachment".to_string(),
    }
}

#[endpoint(
    tags("Dynamic data"),
    status_codes(200, 400, 500),
    description = "Returns random bytes."
)]
async fn bytes_simple(param: BytesSimpleRequest, res: &mut Response) -> Result<(), Error> {
    let n = param.n.min(MAX_BYTES) as usize;
    let mut rng: ChaCha20Rng = match param.seed {
        Some(seed) => ChaCha20Rng::seed_from_u64(seed),
        None => rand::make_rng(),
    };
    let mut bytes: Vec<u8> = vec![0; n];
    rng.fill_bytes(&mut bytes);
    res.add_header(CONTENT_TYPE, "application/octet-stream", true)?;
    res.add_header(CONTENT_LENGTH, n, true)?;
    res.add_header(
        CONTENT_DISPOSITION,
        disposition_header(param.filename),
        true,
    )?;
    res.write_body(bytes)?;
    Ok(())
}

#[endpoint(
    tags("Dynamic data"),
    status_codes(200, 400, 500),
    description = "Streams random bytes."
)]
async fn bytes_stream(param: BytesStreamRequest, res: &mut Response) -> Result<(), Error> {
    let mut rng: ChaCha20Rng = match param.seed {
        Some(seed) => ChaCha20Rng::seed_from_u64(seed),
        None => rand::make_rng(),
    };
    let n = param.n.min(MAX_BYTES) as usize;
    let mut remaining = n;
    let chk = match param.chunk_size {
        Some(chunk_size) => {
            if chunk_size > 0 {
                chunk_size
            } else {
                DEFAULT_CHUNK_SIZE
            }
        }
        None => DEFAULT_CHUNK_SIZE,
    };
    res.add_header(CONTENT_TYPE, "application/octet-stream", true)?;
    res.add_header(CONTENT_LENGTH, n, true)?;
    res.add_header(
        CONTENT_DISPOSITION,
        disposition_header(param.filename),
        true,
    )?;
    res.stream(stream! {
        while remaining > 0 {
            let mut bytes = vec![0; remaining.min(chk)];
            rng.fill_bytes(&mut bytes);
            remaining -= bytes.len();
            yield Ok::<_, Infallible>(bytes);
        }
    });
    Ok(())
}

#[endpoint(
    tags("Dynamic data"),
    status_codes(200),
    description = "Generates a UUID v4."
)]
async fn uuid_v4() -> Json<UuidResponse> {
    let uuid = uuid::Uuid::new_v4();
    Json(UuidResponse {
        uuid: uuid.to_string(),
    })
}

#[endpoint(
    tags("Dynamic data"),
    status_codes(200),
    description = "Generates a UUID v7."
)]
async fn uuid_v7() -> Json<UuidResponse> {
    let uuid = uuid::Uuid::now_v7();
    Json(UuidResponse {
        uuid: uuid.to_string(),
    })
}

#[endpoint(
    tags("Dynamic data"),
    status_codes(200),
    description = "Generates a ULID."
)]
async fn ulid_() -> Json<UlidResponse> {
    let ulid = ulid::Ulid::generate();
    Json(UlidResponse {
        ulid: ulid.to_string(),
    })
}

async fn _delay(param: DelayRequest, req: &mut Request) -> Result<Json<PostResponse>, StatusError> {
    let sec = param.seconds.min(10);
    if sec > 0 {
        tokio::time::sleep(std::time::Duration::from_secs(sec)).await;
    }
    post(req).await
}

#[endpoint(
    tags("Dynamic data"),
    status_codes(200),
    description = "Delays, then echoes a GET request."
)]
async fn delay_get(
    param: DelayRequest,
    req: &mut Request,
) -> Result<Json<PostResponse>, StatusError> {
    _delay(param, req).await
}

#[endpoint(
    tags("Dynamic data"),
    status_codes(200),
    description = "Delays, then echoes a POST request."
)]
async fn delay_post(
    param: DelayRequest,
    req: &mut Request,
) -> Result<Json<PostResponse>, StatusError> {
    _delay(param, req).await
}

#[endpoint(
    tags("Dynamic data"),
    status_codes(200),
    description = "Delays, then echoes a PUT request."
)]
async fn delay_put(
    param: DelayRequest,
    req: &mut Request,
) -> Result<Json<PostResponse>, StatusError> {
    _delay(param, req).await
}

#[endpoint(
    tags("Dynamic data"),
    status_codes(200),
    description = "Delays, then echoes a DELETE request."
)]
async fn delay_delete(
    param: DelayRequest,
    req: &mut Request,
) -> Result<Json<PostResponse>, StatusError> {
    _delay(param, req).await
}

pub fn dynamic_data_router() -> Router {
    Router::with_hoop(remove_slash())
        .push(Router::with_path("bytes/stream/{n}").get(bytes_stream))
        .push(Router::with_path("bytes/{n}").get(bytes_simple))
        .push(Router::with_path("uuid/v4").get(uuid_v4))
        .push(Router::with_path("uuid/v7").get(uuid_v7))
        .push(Router::with_path("ulid").get(ulid_))
        .push(Router::with_path("delay/{seconds}").get(delay_get))
        .push(Router::with_path("delay/{seconds}").post(delay_post))
        .push(Router::with_path("delay/{seconds}").put(delay_put))
        .push(Router::with_path("delay/{seconds}").delete(delay_delete))
}
