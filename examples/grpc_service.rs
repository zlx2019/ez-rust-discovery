use ez_rust_discovery::{ServeOptions, ServiceManager};

/// [NACOS_ADDR] Nacos server address
/// [NACOS_NAMESPACE] Service namespace
/// [SERVICE_ADDR] Worker service address
/// [SERVICE_NAME] Worker service name
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO Start gRPC or HTTP service.

    let manager = ServiceManager::new(ServeOptions::default())?;
    if let Err(e) = manager.online() {
        println!("online fail, caused by: {}", e);
        std::process::exit(1);
    };

    std::thread::sleep(std::time::Duration::from_secs(30));
    // TODO block Listen signal...

    // Go offline before the gRPC or HTTP service shuts down
    manager.offline()?;

    // TODO Close gRPC or HTTP service.
    Ok(())
}