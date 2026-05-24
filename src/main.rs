mod client;
mod server;

use client::RegistryClient;
use rmcp::{ServiceExt, transport::stdio};
use server::PackageRegistryServer;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("info".parse().unwrap())).init();
    let client = Arc::new(RegistryClient::new());
    let server = PackageRegistryServer { client };
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
