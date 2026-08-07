use salvo::oapi::extract::*;
use salvo::prelude::*;
use salvo::trailing_slash::remove_slash;

use crate::rfc9457::ApiResult;

fn _common(code: u16) -> ApiResult<StatusCode> {
    Ok(StatusCode::from_u16(code)
        .map_err(|_| StatusError::bad_request().brief("status code must be between 100 and 999"))?)
}

#[endpoint(
    tags("Status codes"),
    status_codes(200, 400),
    description = "Returns the requested status code for GET."
)]
async fn get(code: PathParam<u16>) -> ApiResult<StatusCode> {
    _common(code.into_inner())
}

#[endpoint(
    tags("Status codes"),
    status_codes(200, 400),
    description = "Returns the requested status code for POST."
)]
async fn post(code: PathParam<u16>) -> ApiResult<StatusCode> {
    _common(code.into_inner())
}

#[endpoint(
    tags("Status codes"),
    status_codes(200, 400),
    description = "Returns the requested status code for DELETE."
)]
async fn delete(code: PathParam<u16>) -> ApiResult<StatusCode> {
    _common(code.into_inner())
}

#[endpoint(
    tags("Status codes"),
    status_codes(200, 400),
    description = "Returns the requested status code for PUT."
)]
async fn put(code: PathParam<u16>) -> ApiResult<StatusCode> {
    _common(code.into_inner())
}

#[endpoint(
    tags("Status codes"),
    status_codes(200, 400),
    description = "Returns the requested status code for PATCH."
)]
async fn patch(code: PathParam<u16>) -> ApiResult<StatusCode> {
    _common(code.into_inner())
}

#[endpoint(
    tags("Status codes"),
    status_codes(200, 400),
    description = "Returns the requested status code for HEAD."
)]
async fn head(code: PathParam<u16>) -> ApiResult<StatusCode> {
    _common(code.into_inner())
}

#[endpoint(
    tags("Status codes"),
    status_codes(200, 400),
    description = "Returns the requested status code for OPTIONS."
)]
async fn options(code: PathParam<u16>) -> ApiResult<StatusCode> {
    _common(code.into_inner())
}

#[endpoint(
    tags("Status codes"),
    status_codes(200, 400),
    description = "Returns the requested status code for QUERY."
)]
async fn query(code: PathParam<u16>) -> ApiResult<StatusCode> {
    _common(code.into_inner())
}

pub fn status_codes_router() -> Router {
    Router::with_hoop(remove_slash())
        .push(Router::with_path("status/get/{code}").get(get))
        .push(Router::with_path("status/post/{code}").post(post))
        .push(Router::with_path("status/delete/{code}").delete(delete))
        .push(Router::with_path("status/put/{code}").put(put))
        .push(Router::with_path("status/patch/{code}").patch(patch))
        .push(Router::with_path("status/head/{code}").head(head))
        .push(Router::with_path("status/options/{code}").options(options))
        .push(Router::with_path("status/query/{code}").query(query))
}
