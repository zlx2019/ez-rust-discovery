use ez_discovery::{ServeOptions, ServiceManager};

/// [NACOS_ADDR] Nacos server address
/// [NACOS_NAMESPACE] Service namespace
/// [SERVICE_ADDR] Worker service address
/// [SERVICE_NAME] Worker service name
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // First initialize the log collector（if you need）
    // TODO Start HTTP service or gRPC service...

    let manager = ServiceManager::new(ServeOptions::default())?;
    if let Err(e) = manager.online() {
        println!("online fail, caused by: {}", e);
        std::process::exit(1);
    };

    // Wait for graceful exit...
    manager.offline()?;
    Ok(())

}