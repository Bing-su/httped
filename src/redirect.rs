use salvo::prelude::*;
use salvo::trailing_slash::remove_slash;
use serde::Deserialize;

#[derive(Deserialize, ToParameters, Debug)]
struct RedirectToQueryParams {
    /// Redirect target URL.
    url: String,
    /// Redirect status code.
    status_code: Option<u16>,
}

#[derive(Deserialize, ToParameters, Debug)]
struct RedirectPathParams {
    /// Number of redirects.
    #[salvo(parameter(minimum = 1, parameter_in = "path"))]
    n: u8,
}

fn _redirect_to(param: RedirectToQueryParams, res: &mut Response) -> Result<(), StatusError> {
    let url = param.url;

    let code = match param.status_code {
        Some(x) if (300..400).contains(&x) => {
            StatusCode::from_u16(x).expect("Status code 300-399 is valid")
        }
        _ => StatusCode::FOUND,
    };
    let redirect = Redirect::with_status_code(code, &url)
        .map_err(|_| StatusError::bad_request().brief("Invalid URL"))?;
    res.render(redirect);
    Ok(())
}

#[endpoint(
    tags("Redirects"),
    status_codes(200, 400, 500),
    description = "Redirects a GET request."
)]
async fn redirect_to_get(
    param: RedirectToQueryParams,
    res: &mut Response,
) -> Result<(), StatusError> {
    _redirect_to(param, res)
}

#[endpoint(
    tags("Redirects"),
    status_codes(200, 400, 500),
    description = "Redirects a POST request."
)]
async fn redirect_to_post(
    param: RedirectToQueryParams,
    res: &mut Response,
) -> Result<(), StatusError> {
    _redirect_to(param, res)
}

#[endpoint(
    tags("Redirects"),
    status_codes(200, 400, 500),
    description = "Redirects a DELETE request."
)]
async fn redirect_to_delete(
    param: RedirectToQueryParams,
    res: &mut Response,
) -> Result<(), StatusError> {
    _redirect_to(param, res)
}

#[endpoint(
    tags("Redirects"),
    status_codes(200, 400, 500),
    description = "Redirects a PUT request."
)]
async fn redirect_to_put(
    param: RedirectToQueryParams,
    res: &mut Response,
) -> Result<(), StatusError> {
    _redirect_to(param, res)
}

#[endpoint(
    tags("Redirects"),
    status_codes(200, 400, 500),
    description = "Redirects a PATCH request."
)]
async fn redirect_to_patch(
    param: RedirectToQueryParams,
    res: &mut Response,
) -> Result<(), StatusError> {
    _redirect_to(param, res)
}

#[endpoint(
    tags("Redirects"),
    status_codes(200, 400, 500),
    description = "Redirects a HEAD request."
)]
async fn redirect_to_head(
    param: RedirectToQueryParams,
    res: &mut Response,
) -> Result<(), StatusError> {
    _redirect_to(param, res)
}

#[endpoint(
    tags("Redirects"),
    status_codes(200, 400, 500),
    description = "Redirects an OPTIONS request."
)]
async fn redirect_to_options(
    param: RedirectToQueryParams,
    res: &mut Response,
) -> Result<(), StatusError> {
    _redirect_to(param, res)
}

#[endpoint(
    tags("Redirects"),
    status_codes(200, 400, 500),
    description = "Creates an absolute redirect chain."
)]
async fn absolute_redirect(
    param: RedirectPathParams,
    req: &mut Request,
    res: &mut Response,
) -> Result<(), StatusError> {
    let n = param.n;
    if n == 0 {
        return Err(StatusError::bad_request().brief("n must be greater than 0"));
    }

    let uri = req.uri().to_string();
    let base = uri
        .split_once('?')
        .map_or(uri.as_str(), |(path, _)| path)
        .strip_suffix(&format!("/absolute-redirect/{n}"))
        .unwrap_or_default();
    let next = if n == 1 {
        format!("{base}/get")
    } else {
        format!("{base}/absolute-redirect/{}", n - 1)
    };
    res.render(Redirect::found(next));
    Ok(())
}

#[endpoint(
    tags("Redirects"),
    status_codes(200, 400, 500),
    description = "Creates a relative redirect chain."
)]
async fn relative_redirect(
    param: RedirectPathParams,
    req: &mut Request,
    res: &mut Response,
) -> Result<(), StatusError> {
    let n = param.n;
    if n == 0 {
        return Err(StatusError::bad_request().brief("n must be greater than 0"));
    }

    let base = req
        .uri()
        .path()
        .strip_suffix(&format!("/relative-redirect/{n}"))
        .unwrap_or_default();
    let next = if n == 1 {
        format!("{base}/get")
    } else {
        format!("{base}/relative-redirect/{}", n - 1)
    };
    res.render(Redirect::found(next));
    Ok(())
}

pub fn redirect_router() -> Router {
    Router::with_hoop(remove_slash())
        .push(
            Router::with_path("redirect-to")
                .get(redirect_to_get)
                .post(redirect_to_post)
                .delete(redirect_to_delete)
                .put(redirect_to_put)
                .patch(redirect_to_patch)
                .head(redirect_to_head)
                .options(redirect_to_options),
        )
        .push(Router::with_path("absolute-redirect/{n}").get(absolute_redirect))
        .push(Router::with_path("relative-redirect/{n}").get(relative_redirect))
}

#[cfg(test)]
mod tests {
    use rstest::*;
    use salvo::{http::header::LOCATION, test::TestClient};

    use super::*;

    #[rstest]
    #[case::without_query(
        "https://localhost:8443/api/absolute-redirect/2",
        "http://localhost/api/relative-redirect/1"
    )]
    #[case::with_query(
        "https://localhost:8443/api/absolute-redirect/2?foo=bar",
        "http://localhost/api/relative-redirect/1?foo=bar"
    )]
    #[tokio::test]
    async fn redirects_preserve_mount_and_semantics(#[case] abs: &str, #[case] rel: &str) {
        let service = Service::new(Router::with_path("api").push(redirect_router()));

        let absolute = TestClient::get(abs).send(&service).await;
        let relative = TestClient::get(rel).send(&service).await;

        assert_eq!(
            absolute.headers()[LOCATION],
            "https://localhost:8443/api/absolute-redirect/1"
        );
        assert_eq!(relative.headers()[LOCATION], "/api/get");
    }

    #[endpoint]
    async fn _get(res: &mut Response) {
        res.render("ok");
    }

    #[tokio::test]
    async fn redirect_to_get_with_status_code() {
        let service = Service::new(redirect_router().push(Router::with_path("겟").get(_get)));

        for code in 300..400 {
            let response = TestClient::get(&format!(
                "http://localhost/redirect-to?url=http%3A%2F%2Flocalhost%2F%EA%B2%9F&status_code={code}"
            ))
            .send(&service)
            .await;

            assert_eq!(
                response.status_code,
                Some(StatusCode::from_u16(code).unwrap())
            );
        }
    }

    #[tokio::test]
    async fn redirect_to_post_with_status_code() {
        let service = Service::new(redirect_router().push(Router::with_path("🤗").post(_get)));

        for code in 300..400 {
            let response = TestClient::post(&format!(
                "http://localhost/redirect-to?url=http://localhost/🤗&status_code={code}"
            ))
            .send(&service)
            .await;

            assert_eq!(
                response.status_code,
                Some(StatusCode::from_u16(code).unwrap())
            );
        }
    }
}
