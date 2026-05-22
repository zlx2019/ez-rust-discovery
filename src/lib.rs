#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![doc = "Ergonomic service discovery client for Nacos, optimized for gRPC/HTTP workloads."]
#![doc = ""]
#![doc = "See the project [README](https://github.com/zlx2019/ez-rust-discovery) for usage examples."]

mod config;
mod error;
mod manager;

pub use config::{
    DEFAULT_GROUP, DEFAULT_WEIGHT, META_GRPC_PORT, ServiceConfig, ServiceConfigBuilder, env_keys,
};
pub use error::{Error, Result};
pub use manager::ServiceManager;
