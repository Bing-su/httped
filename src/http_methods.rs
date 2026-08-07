use salvo::prelude::*;

use crate::helper::{GetResponse, PostResponse, get as helper_get, post as helper_post};

#[endpoint(
    tags("Http methods"),
    status_codes(200, 400, 500),
    description = "Returns request information."
)]
async fn get(req: &mut Request) -> Json<GetResponse> {
    helper_get(req)
}

#[endpoint(
    tags("Http methods"),
    status_codes(200, 400, 500),
    description = "Echoes a POST request."
)]
pub async fn post(req: &mut Request) -> Result<Json<PostResponse>, StatusError> {
    helper_post(req).await
}

#[endpoint(
    tags("Http methods"),
    status_codes(200, 400, 500),
    description = "Echoes a DELETE request."
)]
pub async fn delete(req: &mut Request) -> Result<Json<PostResponse>, StatusError> {
    helper_post(req).await
}

#[endpoint(
    tags("Http methods"),
    status_codes(200, 400, 500),
    description = "Echoes a PUT request."
)]
pub async fn put(req: &mut Request) -> Result<Json<PostResponse>, StatusError> {
    helper_post(req).await
}

#[endpoint(
    tags("Http methods"),
    status_codes(200, 400, 500),
    description = "Echoes a PATCH request."
)]
pub async fn patch(req: &mut Request) -> Result<Json<PostResponse>, StatusError> {
    helper_post(req).await
}

#[endpoint(
    tags("Http methods"),
    status_codes(200, 400, 500),
    description = "Returns request information without a body."
)]
pub async fn head(req: &mut Request) -> Json<GetResponse> {
    helper_get(req)
}

#[endpoint(
    tags("Http methods"),
    status_codes(200, 400, 500),
    description = "Echoes an OPTIONS request."
)]
pub async fn options(req: &mut Request) -> Result<Json<PostResponse>, StatusError> {
    helper_post(req).await
}

#[endpoint(
    tags("Http methods"),
    status_codes(200, 400, 500),
    description = "Echoes a QUERY request."
)]
pub async fn query(req: &mut Request) -> Result<Json<PostResponse>, StatusError> {
    helper_post(req).await
}

pub fn http_methods_router() -> Router {
    Router::new()
        .push(Router::with_path("get").get(get))
        .push(Router::with_path("post").post(post))
        .push(Router::with_path("delete").delete(delete))
        .push(Router::with_path("put").put(put))
        .push(Router::with_path("patch").patch(patch))
        .push(Router::with_path("head").head(head))
        .push(Router::with_path("options").options(options))
        .push(Router::with_path("query").query(query))
}
