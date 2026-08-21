//! REST API binary entrypoint.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    oracle_server::run().await
}
