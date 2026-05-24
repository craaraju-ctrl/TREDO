#[tokio::main]
async fn main() {
    arkm_server::start_server().await;
}
