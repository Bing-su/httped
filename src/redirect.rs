use anyhow::Result;
use salvo::oapi::extract::PathParam;
use salvo::prelude::*;
use salvo::trailing_slash::remove_slash;
use serde::Deserialize;

#[derive(Deserialize, ToParameters, Debug)]
struct RedirectToQueryParams {
    url: String,
    status_code: Option<u16>,
}

fn _redirect_to(param: RedirectToQueryParams, res: &mut Response) -> Result<()> {
    let url = param.url;

    let code = match param.status_code {
        Some(x) if (300..400).contains(&x) => StatusCode::from_u16(x)?,
        _ => StatusCode::FOUND,
    };
    res.render(Redirect::with_status_code(code, &url)?);
    Ok(())
}

#[endpoint(tags("Redirects"), status_codes(200, 400, 500))]
async fn redirect_to_get(param: RedirectToQueryParams, res: &mut Response) -> Result<()> {
    _redirect_to(param, res)
}

#[endpoint(tags("Redirects"), status_codes(200, 400, 500))]
async fn redirect_to_post(param: RedirectToQueryParams, res: &mut Response) -> Result<()> {
    _redirect_to(param, res)
}

#[endpoint(tags("Redirects"), status_codes(200, 400, 500))]
async fn redirect_to_delete(param: RedirectToQueryParams, res: &mut Response) -> Result<()> {
    _redirect_to(param, res)
}

#[endpoint(tags("Redirects"), status_codes(200, 400, 500))]
async fn redirect_to_put(param: RedirectToQueryParams, res: &mut Response) -> Result<()> {
    _redirect_to(param, res)
}

#[endpoint(tags("Redirects"), status_codes(200, 400, 500))]
async fn redirect_to_patch(param: RedirectToQueryParams, res: &mut Response) -> Result<()> {
    _redirect_to(param, res)
}

#[endpoint(tags("Redirects"), status_codes(200, 400, 500))]
async fn redirect_to_head(param: RedirectToQueryParams, res: &mut Response) -> Result<()> {
    _redirect_to(param, res)
}

#[endpoint(tags("Redirects"), status_codes(200, 400, 500))]
async fn redirect_to_options(param: RedirectToQueryParams, res: &mut Response) -> Result<()> {
    _redirect_to(param, res)
}

#[endpoint(tags("Redirects"), status_codes(200, 400, 500))]
async fn absolute_redirect(n: PathParam<u8>, res: &mut Response) -> salvo::Result<()> {
    let n = n.into_inner();
    if n == 0 {
        return Err(salvo::Error::HttpStatus(
            StatusError::bad_request().brief("n must be greater than 0"),
        ));
    }

    if n == 1 {
        res.render(Redirect::found("/get"));
        return Ok(());
    }

    let next = format!("{}", n - 1);
    res.render(Redirect::found(next));
    Ok(())
}

#[endpoint(tags("Redirects"), status_codes(200, 400, 500))]
async fn relative_redirect(n: PathParam<u8>, res: &mut Response) -> salvo::Result<()> {
    let n = n.into_inner();
    if n == 0 {
        return Err(salvo::Error::HttpStatus(
            StatusError::bad_request().brief("n must be greater than 0"),
        ));
    }

    res.status_code(StatusCode::FOUND);
    if n == 1 {
        res.add_header("Location", "/get", true)?;
        return Ok(());
    }

    let next = format!("{}", n - 1);
    res.add_header("Location", &next, true)?;
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
