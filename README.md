## Install
```shell
cargo add ez-rust-discovery
```

## Usage Example
```rust
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
```

## Environment
| Name                | Description                                   | Default value |
| ------------------- | --------------------------------------------- | ------------- |
| **NACOS_ADDR**      | Nacos server address                          |               |
| **NACOS_NAMESPACE** | Service namespace                             | public        |
| **SERVICE_ADDR**    | Service address                               |               |
| **SERVICE_NAME**    | Service name                                  |               |
| **NACOS_USERNAME**  | Nacos auth username (optional, paired w/ pwd) |               |
| **NACOS_PASSWORD**  | Nacos auth password (optional, paired w/ usr) |               |

> `NACOS_USERNAME` and `NACOS_PASSWORD` must be provided together, or both omitted.


