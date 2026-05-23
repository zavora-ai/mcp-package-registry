mod client;
mod server;

use client::RegistryClient;
use rmcp::{ServiceExt, transport::stdio};
use server::PackageRegistryServer;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = Arc::new(RegistryClient::new());
    let server = PackageRegistryServer { client };
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
