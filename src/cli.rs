use std::time::Duration;

use clap::Parser;
use salvo::oapi::ToSchema;
use salvo::prelude::*;
use salvo::serve_static::static_embed;
use serde::Serialize;

use crate::auth;
use crate::dynamic_data;
use crate::helper::Asset;
use crate::http_methods;
use crate::redirect;
use crate::request_inspection;
use crate::sse;
use crate::status_codes;
use crate::websocket;

const PROJECT_NAME: &str = env!("CARGO_PKG_NAME");
const PROJECT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// 🌐 Host to bind the server to
    #[arg(short('H'), long, default_value = "localhost", env = "HTTPED_HOST")]
    pub host: String,

    /// 🔌 Port to bind the server to
    #[arg(short, long, default_value_t = 8698, env = "HTTPED_PORT")]
    pub port: u16,
}

#[derive(Serialize, ToSchema, Debug)]
struct ResponseHealth {
    message: String,
}

#[endpoint(description = "Checks service health.")]
async fn health() -> Json<ResponseHealth> {
    Json(ResponseHealth {
        message: "ok".to_string(),
    })
}

pub fn build_router() -> Router {
    let router = Router::new()
        .push(Router::with_path("health").get(health))
        .push(http_methods::http_methods_router())
        .push(auth::auth_router())
        .push(request_inspection::request_inspection_router())
        .push(status_codes::status_codes_router())
        .push(redirect::redirect_router())
        .push(websocket::ws_router())
        .push(sse::sse_router())
        .push(dynamic_data::dynamic_data_router())
        .push(Router::with_path("asset/{*path}").get(static_embed::<Asset>()));

    let doc = OpenApi::new(PROJECT_NAME, PROJECT_VERSION).merge_router(&router);

    let openapi = Scalar::new("openapi.json").lib_url("asset/scalar.js");

    router
        .unshift(doc.into_router("openapi.json"))
        .push(Router::new().goal(openapi))
}

pub fn build_service() -> Service {
    let router = build_router();

    Service::new(router)
        .hoop(Logger::new())
        .hoop(Compression::new())
}

pub async fn entry() -> Result<(), String> {
    tracing_subscriber::fmt().init();

    let args = Cli::parse();
    let service = build_service();

    let acceptor = TcpListener::new((args.host.clone(), args.port))
        .try_bind()
        .await;
    match acceptor {
        Ok(acceptor) => {
            let server = Server::new(acceptor);
            let handle = server.handle();

            tokio::spawn(async move {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to listen for Ctrl+C");
                handle.stop_graceful(Some(Duration::from_secs(5)));
            });

            server.serve(service).await;
        }
        Err(e) => {
            let msg = format!("Failed to bind to {}:{} - {}", args.host, args.port, e);
            tracing::error!("{}", msg);
            return Err(msg);
        }
    }
    Ok(())
}
