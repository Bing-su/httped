use std::sync::LazyLock;

use anyhow::Result;
use regex::Regex;
use salvo::Error;
use salvo::basic_auth::{BasicAuth, BasicAuthValidator};
use salvo::oapi::ToSchema;
use salvo::oapi::extract::*;
use salvo::prelude::*;
use salvo::trailing_slash::remove_slash;
use serde::Serialize;

const BEARER_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^Bearer\s+(?P<token>[A-Za-z0-9\-\._~\+/]+=*)$").expect("Must be a valid regex")
});

#[handler]
async fn save_username_password(
    username: PathParam<String>,
    password: PathParam<String>,
    depot: &mut Depot,
) {
    let username = username.into_inner();
    let password = password.into_inner();
    depot.insert("username", username);
    depot.insert("password", password);
}

struct HttpedBasicAuthValidator;

impl BasicAuthValidator for HttpedBasicAuthValidator {
    async fn validate(&self, username: &str, password: &str, depot: &mut Depot) -> bool {
        let user = depot.get::<String>("username");
        let pass = depot.get::<String>("password");
        match (user, pass) {
            (Ok(u), Ok(p)) => u == username && p == password,
            _ => false,
        }
    }
}

#[derive(Serialize, ToSchema)]
struct BasicAuthResponse {
    authenticated: bool,
    user: String,
}

#[derive(Serialize, ToSchema)]
struct BearerResponse {
    authenticated: bool,
    token: String,
}

#[endpoint(tags("auth"), status_codes(200, 401))]
#[allow(unused_variables)]
async fn basic_auth(
    username: PathParam<String>,
    password: PathParam<String>,
) -> Result<Json<BasicAuthResponse>, Error> {
    let response = BasicAuthResponse {
        authenticated: true,
        user: username.into_inner(),
    };
    Ok(Json(response))
}

#[endpoint(tags("auth"), status_codes(200, 400, 401))]
async fn bearer_auth(
    authorization: HeaderParam<String, false>,
) -> Result<Json<BearerResponse>, Error> {
    let header = authorization.into_inner().ok_or_else(|| {
        Error::HttpStatus(StatusError::unauthorized().brief("Missing authorization header"))
    })?;
    let auth = BEARER_REGEX.captures(&header).ok_or_else(|| {
        Error::HttpStatus(StatusError::bad_request().brief("Invalid authorization header"))
    })?;

    let token = auth["token"].to_string();

    let response = BearerResponse {
        authenticated: true,
        token: token,
    };
    Ok(Json(response))
}

pub fn auth_router() -> Router {
    let auth_handler = BasicAuth::new(HttpedBasicAuthValidator);
    let basic_auth_router = Router::new()
        .hoop(save_username_password)
        .hoop(auth_handler)
        .push(Router::with_path("basic-auth/{username}/{password}").get(basic_auth));
    let bearer_auth_router = Router::new().push(Router::with_path("bearer").get(bearer_auth));
    Router::with_hoop(remove_slash())
        .push(basic_auth_router)
        .push(bearer_auth_router)
}
