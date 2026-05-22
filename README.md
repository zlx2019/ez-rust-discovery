# ez-rust-discovery

[![crates.io](https://img.shields.io/crates/v/ez-rust-discovery.svg)](https://crates.io/crates/ez-rust-discovery)
[![docs.rs](https://docs.rs/ez-rust-discovery/badge.svg)](https://docs.rs/ez-rust-discovery)
[![license](https://img.shields.io/crates/l/ez-rust-discovery.svg)](./LICENSE)

基于 [Nacos](https://nacos.io/) 的轻量级服务发现客户端, 为 gRPC / HTTP 微服务的
注册与下线场景提供开箱即用的 API.

- 类型安全的 builder, 也支持从环境变量加载.
- 一等 `async` API, 同时附带 `*_blocking` 同步辅助方法.
- 完整的 `Error` 类型 (基于 `thiserror`).
- 支持 Nacos HTTP 鉴权.
- 默认写入 `gRPC_port` 元数据, 也允许追加任意自定义元数据.

## 安装

```bash
cargo add ez-rust-discovery
```

`Cargo.toml`:

```toml
[dependencies]
ez-rust-discovery = "0.2"
tokio = { version = "1", features = ["full"] }
```

## 快速开始

### 链式 builder

```rust,no_run
use ez_rust_discovery::{ServiceConfig, ServiceManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServiceConfig::builder()
        .nacos_addr("127.0.0.1:8848")
        .namespace("public")
        .service_name("payment-service")
        .service_port(9000)
        .metadata("region", "cn-east-1")
        .build()?;

    let manager = ServiceManager::new(config).await?;
    manager.register().await?;

    // ... 启动 gRPC / HTTP 服务 ...

    tokio::signal::ctrl_c().await?;
    manager.deregister().await?;
    Ok(())
}
```

### 从环境变量加载

```rust,no_run
use ez_rust_discovery::{ServiceConfig, ServiceManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = ServiceManager::new(ServiceConfig::from_env()?).await?;
    manager.register().await?;
    tokio::signal::ctrl_c().await?;
    manager.deregister().await?;
    Ok(())
}
```

### 同步场景

无 tokio runtime 的纯同步程序可使用 `*_blocking` 方法:

```rust,no_run
use ez_rust_discovery::{ServiceConfig, ServiceManager};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = ServiceManager::new_blocking(ServiceConfig::from_env()?)?;
    manager.register_blocking()?;

    // ... 业务逻辑 ...

    manager.deregister_blocking()?;
    Ok(())
}
```

> ⚠️ 不要在已有 tokio runtime 的上下文里调用 `*_blocking` 方法, 否则会 panic.

## 环境变量

| 名称              | 必填 | 默认值     | 说明                                          |
| ----------------- | ---- | ---------- | --------------------------------------------- |
| `NACOS_ADDR`      | 是   | -          | Nacos 服务器地址, 格式 `host:port`            |
| `NACOS_NAMESPACE` | 是   | -          | 命名空间 ID                                   |
| `SERVICE_ADDR`    | 是   | -          | 监听地址 (仅 port 部分被使用)                 |
| `SERVICE_NAME`    | 是   | -          | 服务名                                        |
| `SERVICE_HOST`    | 否   | 本机 IP    | 注册到 Nacos 的对外 host                      |
| `NACOS_USERNAME`  | 否   | -          | 鉴权用户名 (必须与 `NACOS_PASSWORD` 同时提供) |
| `NACOS_PASSWORD`  | 否   | -          | 鉴权密码 (必须与 `NACOS_USERNAME` 同时提供)   |

## 运行示例

```bash
export NACOS_ADDR=127.0.0.1:8848
export NACOS_NAMESPACE=public
export SERVICE_ADDR=0.0.0.0:9000
export SERVICE_NAME=demo-grpc
cargo run --example grpc_service
```

## 兼容性

- Rust `1.85+` (edition 2024).
- `nacos-sdk` `0.8.x`.

## License

[MIT](./LICENSE)
