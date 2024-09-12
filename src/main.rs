use lib_trebuchet::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    cli::run().await
}
