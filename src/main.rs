use salvo::oapi::ToSchema;
use salvo::prelude::*;
use salvo::serve_static::static_embed;
use serde::Serialize;

use httped::auth;
use httped::helper::Asset;
use httped::http_methods;
use httped::redirect;
use httped::request_inspection;
use httped::status_codes;

#[derive(Serialize, ToSchema, Debug)]
struct ResponseHealth {
    message: String,
}

#[endpoint]
async fn health() -> Json<ResponseHealth> {
    Json(ResponseHealth {
        message: "ok".to_string(),
    })
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let router = Router::new()
        .push(Router::with_path("health").get(health))
        .push(http_methods::http_methods_router())
        .push(auth::auth_router())
        .push(request_inspection::request_inspection_router())
        .push(status_codes::status_codes_router())
        .push(redirect::redirect_router())
        .push(Router::with_path("asset/{*path}").get(static_embed::<Asset>()));

    let doc = OpenApi::new("Api", "0.1.0").merge_router(&router);

    let openapi = Scalar::new("openapi.json").lib_url("asset/scalar.js");
    let router = router
        .unshift(doc.into_router("openapi.json"))
        .push(Router::new().goal(openapi));

    let service = Service::new(router)
        .hoop(Logger::new())
        .hoop(Compression::new());

    let acceptor = TcpListener::new("127.0.0.1:8698").bind().await;
    Server::new(acceptor).serve(service).await;
}
