use httped::cli::entry;

#[tokio::main]
async fn main() {
    if let Err(e) = entry().await {
        eprintln!("{}", e);
    }
}
