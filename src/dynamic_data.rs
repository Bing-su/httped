use std::convert::Infallible;

use anyhow::Result;
use async_stream::stream;
use rand::prelude::*;
use rand::rngs::ChaCha20Rng;
use salvo::prelude::*;
use salvo::trailing_slash::remove_slash;
use serde::{Deserialize, Serialize};

const DEFAULT_CHUNK_SIZE: usize = 8 * 1024;
const MAX_BYTES: u32 = 1000000; // 1 MB

#[derive(Serialize, Deserialize, ToParameters, ToSchema, Debug)]
struct BytesSimpleRequest {
    #[salvo(parameter(parameter_in = "path", minimum = 0, maximum = 1000000))]
    n: u32,
    #[salvo(parameter(parameter_in = "query", minimum = 0))]
    seed: Option<u64>,
}

#[derive(Serialize, Deserialize, ToParameters, ToSchema, Debug)]
struct BytesStreamRequest {
    #[salvo(parameter(parameter_in = "path", minimum = 0, maximum = 1000000))]
    n: u32,
    #[salvo(parameter(parameter_in = "query", minimum = 0))]
    seed: Option<u64>,
    #[salvo(parameter(parameter_in = "query", minimum = 1, default = 8192))]
    chunk_size: Option<usize>,
}

#[derive(Serialize, ToSchema, Debug)]
struct UuidResponse {
    uuid: String,
}

#[derive(Serialize, ToSchema, Debug)]
struct UlidResponse {
    ulid: String,
}

#[endpoint(tags("Dynamic data"), status_codes(200, 400, 500))]
async fn bytes_simple(param: BytesSimpleRequest, res: &mut Response) -> Result<()> {
    let n = param.n.min(MAX_BYTES) as usize;
    let mut rng: ChaCha20Rng = match param.seed {
        Some(seed) => ChaCha20Rng::seed_from_u64(seed),
        None => rand::make_rng(),
    };
    let mut bytes: Vec<u8> = vec![0; n];
    rng.fill_bytes(&mut bytes);
    res.add_header("Content-Type", "application/octet-stream", true)?;
    res.add_header("Content-Length", n, true)?;
    res.write_body(bytes)?;
    Ok(())
}

#[endpoint(tags("Dynamic data"), status_codes(200, 400, 500))]
async fn bytes_stream(param: BytesStreamRequest, res: &mut Response) -> Result<()> {
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
    res.add_header("Content-Type", "application/octet-stream", true)?;
    res.add_header("Content-Length", n, true)?;
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

#[endpoint(tags("Dynamic data"), status_codes(200))]
async fn uuid_v4() -> Json<UuidResponse> {
    let uuid = uuid::Uuid::new_v4();
    Json(UuidResponse {
        uuid: uuid.to_string(),
    })
}

#[endpoint(tags("Dynamic data"), status_codes(200))]
async fn uuid_v7() -> Json<UuidResponse> {
    let uuid = uuid::Uuid::now_v7();
    Json(UuidResponse {
        uuid: uuid.to_string(),
    })
}

#[endpoint(tags("Dynamic data"), status_codes(200))]
async fn ulid_() -> Json<UlidResponse> {
    let ulid = ulid::Ulid::generate();
    Json(UlidResponse {
        ulid: ulid.to_string(),
    })
}

pub fn dynamic_data_router() -> Router {
    Router::with_hoop(remove_slash())
        .push(Router::with_path("bytes/stream/{n}").get(bytes_stream))
        .push(Router::with_path("bytes/{n}").get(bytes_simple))
        .push(Router::with_path("uuid/v4").get(uuid_v4))
        .push(Router::with_path("uuid/v7").get(uuid_v7))
        .push(Router::with_path("ulid").get(ulid_))
}
