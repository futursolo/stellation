#[tokio::main]
async fn main() -> anyhow::Result<()> {
    stackctl::main().await
}
