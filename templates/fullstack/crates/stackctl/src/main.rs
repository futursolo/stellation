// A vendored version of stackctl, this is a workaround until https://github.com/rust-lang/rfcs/pull/3168 is implemented.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    stackable_cli::main().await
}
