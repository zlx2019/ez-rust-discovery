// Integration tests against a real Nacos instance; skipped by default via `#[ignore]`.
//
// Run explicitly: `cargo test -- --ignored`.

#![allow(clippy::unwrap_used, clippy::expect_used, missing_docs)]

use std::time::Duration;

use ez_rust_discovery::{ServiceConfig, ServiceManager};

/// Build a full configuration via the builder and exercise register/deregister end-to-end.
#[tokio::test]
#[ignore = "requires a running Nacos server"]
async fn register_and_deregister_against_local_nacos() {
    let config = ServiceConfig::builder()
        .nacos_addr("192.168.14.121:8848")
        .namespace("public")
        .service_name("ez-rust-discovery-it")
        .service_port(19999)
        .build()
        .expect("config build failed");

    let manager = ServiceManager::new(config)
        .await
        .expect("manager init failed");
    manager.register().await.expect("register failed");
    tokio::time::sleep(Duration::from_secs(10)).await;
    manager.deregister().await.expect("deregister failed");
}
