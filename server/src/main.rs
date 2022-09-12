use server::server::TcpBuilder;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    TcpBuilder::from_env()?.run().await?;
    Ok(())
}
