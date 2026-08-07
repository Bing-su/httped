use salvo::oapi::extract::*;
use salvo::prelude::*;
use serde::Serialize;

#[derive(Serialize, ToSchema)]
struct StatusCodeResponse {
    code: u16,
}

fn _common(res: &mut Response, code: u16) -> Result<Json<StatusCodeResponse>, StatusError> {
    res.status_code(StatusCode::from_u16(code).map_err(|_| {
        StatusError::bad_request().brief("status code must be between 100 and 999")
    })?);
    let response = StatusCodeResponse { code };
    Ok(Json(response))
}

#[endpoint(
    tags("Status codes"),
    status_codes(200, 400),
    description = "Returns the requested status code for GET."
)]
async fn get(
    res: &mut Response,
    code: PathParam<u16>,
) -> Result<Json<StatusCodeResponse>, StatusError> {
    _common(res, code.into_inner())
}

#[endpoint(
    tags("Status codes"),
    status_codes(200, 400),
    description = "Returns the requested status code for POST."
)]
async fn post(
    res: &mut Response,
    code: PathParam<u16>,
) -> Result<Json<StatusCodeResponse>, StatusError> {
    _common(res, code.into_inner())
}

#[endpoint(
    tags("Status codes"),
    status_codes(200, 400),
    description = "Returns the requested status code for DELETE."
)]
async fn delete(
    res: &mut Response,
    code: PathParam<u16>,
) -> Result<Json<StatusCodeResponse>, StatusError> {
    _common(res, code.into_inner())
}

#[endpoint(
    tags("Status codes"),
    status_codes(200, 400),
    description = "Returns the requested status code for PUT."
)]
async fn put(
    res: &mut Response,
    code: PathParam<u16>,
) -> Result<Json<StatusCodeResponse>, StatusError> {
    _common(res, code.into_inner())
}

#[endpoint(
    tags("Status codes"),
    status_codes(200, 400),
    description = "Returns the requested status code for PATCH."
)]
async fn patch(
    res: &mut Response,
    code: PathParam<u16>,
) -> Result<Json<StatusCodeResponse>, StatusError> {
    _common(res, code.into_inner())
}

#[endpoint(
    tags("Status codes"),
    status_codes(200, 400),
    description = "Returns the requested status code for HEAD."
)]
async fn head(
    res: &mut Response,
    code: PathParam<u16>,
) -> Result<Json<StatusCodeResponse>, StatusError> {
    _common(res, code.into_inner())
}

#[endpoint(
    tags("Status codes"),
    status_codes(200, 400),
    description = "Returns the requested status code for OPTIONS."
)]
async fn options(
    res: &mut Response,
    code: PathParam<u16>,
) -> Result<Json<StatusCodeResponse>, StatusError> {
    _common(res, code.into_inner())
}

#[endpoint(
    tags("Status codes"),
    status_codes(200, 400),
    description = "Returns the requested status code for QUERY."
)]
async fn query(
    res: &mut Response,
    code: PathParam<u16>,
) -> Result<Json<StatusCodeResponse>, StatusError> {
    _common(res, code.into_inner())
}

pub fn status_codes_router() -> Router {
    Router::new()
        .push(Router::with_path("status/get/{code}").get(get))
        .push(Router::with_path("status/post/{code}").post(post))
        .push(Router::with_path("status/delete/{code}").delete(delete))
        .push(Router::with_path("status/put/{code}").put(put))
        .push(Router::with_path("status/patch/{code}").patch(patch))
        .push(Router::with_path("status/head/{code}").head(head))
        .push(Router::with_path("status/options/{code}").options(options))
        .push(Router::with_path("status/query/{code}").query(query))
}
