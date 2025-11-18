use ez_discovery::ServeOptions;

/// [NACOS_ADDR] Nacos server address
/// [NACOS_NAMESPACE] Service namespace
/// [SERVICE_ADDR] Worker service address
/// [SERVICE_NAME] Worker service name
fn main() {
    // First initialize the log collector（if you need）
    // TODO Start HTTP service or gRPC service...

    if let Err(e) = ez_discovery::online(ServeOptions::default()) {
        println!("online fail, caused by: {}", e);
        std::process::exit(1);
    };

    // Wait for graceful exit...
}