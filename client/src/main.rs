use client::ClientBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    ClientBuilder::from_env()?.run().await?;
    Ok(())
}
