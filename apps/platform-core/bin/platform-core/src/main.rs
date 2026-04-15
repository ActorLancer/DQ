#[tokio::main]
async fn main() {
    platform_core::run().await.expect("run platform-core");
}
