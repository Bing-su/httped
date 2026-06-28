use salvo::oapi::extract::*;
use salvo::prelude::*;

#[endpoint]
async fn hello(name: QueryParam<String, false>) -> String {
    format!("Hello, {}!", name.as_deref().unwrap_or_else(|| "World"))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let router = Router::new().push(Router::with_path("hello").get(hello));

    let doc = OpenApi::new("Api", "0.1.0").merge_router(&router);

    let router = router
        .unshift(doc.into_router("/openapi.json"))
        .unshift(Scalar::new("/openapi.json").into_router("/docs"));

    let service = Service::new(router).hoop(Logger::new());

    let acceptor = TcpListener::new("127.0.0.1:8698").bind().await;
    Server::new(acceptor).serve(service).await;
}
