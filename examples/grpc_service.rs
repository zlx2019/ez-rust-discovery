#![allow(missing_docs)]

// Minimal runnable example: register on startup, deregister on Ctrl+C.
//
// Set the following environment variables before running:
//
//     export NACOS_ADDR=127.0.0.1:8848
//     export NACOS_NAMESPACE=public
//     export SERVICE_ADDR=0.0.0.0:9000
//     export SERVICE_NAME=demo-grpc
//     # optional:
//     # export NACOS_USERNAME=nacos
//     # export NACOS_PASSWORD=nacos
//     # export SERVICE_HOST=10.0.0.1
//
// Then:
//
//     cargo run --example grpc_service

use ez_rust_discovery::{ServiceConfig, ServiceManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    // 1. Load configuration from the environment (or use the builder).
    let config = ServiceConfig::from_env()?;

    // 2. Build the manager and register the instance.
    let manager = ServiceManager::new(config).await?;
    manager.register().await?;

    // 3. TODO: start your gRPC / HTTP server here.

    // 4. Block until a termination signal arrives.
    tokio::signal::ctrl_c().await?;

    // 5. Gracefully deregister.
    manager.deregister().await?;
    Ok(())
}
