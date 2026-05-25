#[tokio::main]
async fn main() {
    tredo_server::start_server().await;
}
