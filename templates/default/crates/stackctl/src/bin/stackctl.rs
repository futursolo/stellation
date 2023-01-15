// A vendored version of stctl, this is a workaround until https://github.com/rust-lang/rfcs/pull/3168 is implemented.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    stctl::main().await
}
