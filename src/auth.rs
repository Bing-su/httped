use anyhow::Result;
use salvo::basic_auth::{BasicAuth, BasicAuthValidator};
use salvo::oapi::ToSchema;
use salvo::oapi::extract::*;
use salvo::prelude::*;
use salvo::trailing_slash::remove_slash;
use serde::Serialize;

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

#[endpoint(tags("auth"), status_codes(200, 401))]
#[allow(unused_variables)]
async fn basic_auth(
    username: PathParam<String>,
    password: PathParam<String>,
) -> Result<Json<BasicAuthResponse>> {
    let response = BasicAuthResponse {
        authenticated: true,
        user: username.into_inner(),
    };
    Ok(Json(response))
}

pub fn auth_router() -> Router {
    let auth_handler = BasicAuth::new(HttpedBasicAuthValidator);
    Router::with_hoop(remove_slash())
        .hoop(save_username_password)
        .hoop(auth_handler)
        .push(Router::with_path("/basic-auth/{username}/{password}").get(basic_auth))
}
