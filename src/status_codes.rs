use salvo::Error;
use salvo::oapi::extract::*;
use salvo::prelude::*;
use salvo::trailing_slash::remove_slash;
use serde::Serialize;

#[derive(Serialize, ToSchema)]
struct StatusCodeResponse {
    code: u16,
}

fn _common(res: &mut Response, code: u16) -> Result<Json<StatusCodeResponse>, Error> {
    if !(100..=999).contains(&code) {
        return Err(Error::HttpStatus(
            StatusError::bad_request().brief("status code must be between 100 and 999"),
        ));
    }

    res.status_code(StatusCode::from_u16(code).expect("just checked that code is valid"));
    let response = StatusCodeResponse { code };
    Ok(Json(response))
}

#[endpoint(tags("Status codes"), status_codes(200, 400))]
async fn get(res: &mut Response, code: PathParam<u16>) -> Result<Json<StatusCodeResponse>, Error> {
    _common(res, code.into_inner())
}

#[endpoint(tags("Status codes"), status_codes(200, 400))]
async fn post(res: &mut Response, code: PathParam<u16>) -> Result<Json<StatusCodeResponse>, Error> {
    _common(res, code.into_inner())
}

#[endpoint(tags("Status codes"), status_codes(200, 400))]
async fn delete(
    res: &mut Response,
    code: PathParam<u16>,
) -> Result<Json<StatusCodeResponse>, Error> {
    _common(res, code.into_inner())
}

#[endpoint(tags("Status codes"), status_codes(200, 400))]
async fn put(res: &mut Response, code: PathParam<u16>) -> Result<Json<StatusCodeResponse>, Error> {
    _common(res, code.into_inner())
}

#[endpoint(tags("Status codes"), status_codes(200, 400))]
async fn patch(
    res: &mut Response,
    code: PathParam<u16>,
) -> Result<Json<StatusCodeResponse>, Error> {
    _common(res, code.into_inner())
}

#[endpoint(tags("Status codes"), status_codes(200, 400))]
async fn head(res: &mut Response, code: PathParam<u16>) -> Result<Json<StatusCodeResponse>, Error> {
    _common(res, code.into_inner())
}

#[endpoint(tags("Status codes"), status_codes(200, 400))]
async fn options(
    res: &mut Response,
    code: PathParam<u16>,
) -> Result<Json<StatusCodeResponse>, Error> {
    _common(res, code.into_inner())
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
}
