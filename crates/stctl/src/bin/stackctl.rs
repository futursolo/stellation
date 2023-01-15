#[tokio::main]
async fn main() -> anyhow::Result<()> {
    stctl::main().await
}
