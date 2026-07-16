use httped::cli::entry;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    if let Err(e) = entry().await {
        eprintln!("{}", e);
    }
}
