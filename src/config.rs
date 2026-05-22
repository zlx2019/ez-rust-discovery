use std::collections::HashMap;
use std::env;

use crate::error::{Error, Result};

/// Default service group, equivalent to nacos's `DEFAULT_GROUP`.
pub const DEFAULT_GROUP: &str = "DEFAULT_GROUP";

/// Default instance weight.
pub const DEFAULT_WEIGHT: f64 = 1.0;

/// Metadata key for the gRPC port (used by clients to distinguish multi-protocol endpoints).
pub const META_GRPC_PORT: &str = "gRPC_port";

/// Names of the environment variables consumed by [`ServiceConfig::from_env`].
pub mod env_keys {
    /// Nacos server address (`host:port`).
    pub const NACOS_ADDR: &str = "NACOS_ADDR";
    /// Nacos namespace id.
    pub const NACOS_NAMESPACE: &str = "NACOS_NAMESPACE";
    /// Nacos auth username.
    pub const NACOS_USERNAME: &str = "NACOS_USERNAME";
    /// Nacos auth password.
    pub const NACOS_PASSWORD: &str = "NACOS_PASSWORD";
    /// Local listening address (`host:port`); only the port is used for registration.
    pub const SERVICE_ADDR: &str = "SERVICE_ADDR";
    /// Service name.
    pub const SERVICE_NAME: &str = "SERVICE_NAME";
    /// Advertised host registered to Nacos (defaults to the local IP).
    pub const SERVICE_HOST: &str = "SERVICE_HOST";
}

/// Full configuration required to register a service instance.
///
/// Build one via [`ServiceConfig::builder`] or [`ServiceConfig::from_env`].
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// Nacos server address, formatted as `host:port`.
    pub nacos_addr: String,
    /// Nacos namespace id.
    pub namespace: String,
    /// Service name.
    pub service_name: String,
    /// Service group, defaults to [`DEFAULT_GROUP`].
    pub group: String,
    /// Advertised host (IP or hostname) registered to Nacos.
    pub service_host: String,
    /// Advertised port registered to Nacos.
    pub service_port: u16,
    /// Instance weight, defaults to [`DEFAULT_WEIGHT`].
    pub weight: f64,
    /// Whether the instance is ephemeral, defaults to `true`.
    pub ephemeral: bool,
    /// Auth credentials (`username`, `password`); both must be provided or neither.
    pub auth: Option<(String, String)>,
    /// Extra metadata. [`META_GRPC_PORT`] is auto-populated with the port unless the user
    /// supplies their own value.
    pub metadata: HashMap<String, String>,
}

impl ServiceConfig {
    /// Create a fresh, empty builder.
    pub fn builder() -> ServiceConfigBuilder {
        ServiceConfigBuilder::default()
    }

    /// Load configuration from environment variables.
    ///
    /// Required: [`NACOS_ADDR`](env_keys::NACOS_ADDR), [`NACOS_NAMESPACE`](env_keys::NACOS_NAMESPACE),
    /// [`SERVICE_ADDR`](env_keys::SERVICE_ADDR), [`SERVICE_NAME`](env_keys::SERVICE_NAME).
    ///
    /// Optional: [`SERVICE_HOST`](env_keys::SERVICE_HOST) (falls back to the local IP),
    /// [`NACOS_USERNAME`](env_keys::NACOS_USERNAME) + [`NACOS_PASSWORD`](env_keys::NACOS_PASSWORD)
    /// (both must be present, or both absent).
    pub fn from_env() -> Result<Self> {
        let nacos_addr = read_env(env_keys::NACOS_ADDR)?;
        let namespace = read_env(env_keys::NACOS_NAMESPACE)?;
        let service_addr = read_env(env_keys::SERVICE_ADDR)?;
        let service_name = read_env(env_keys::SERVICE_NAME)?;
        let service_host = env::var(env_keys::SERVICE_HOST).ok();
        let username = env::var(env_keys::NACOS_USERNAME).ok();
        let password = env::var(env_keys::NACOS_PASSWORD).ok();

        let mut builder = Self::builder()
            .nacos_addr(nacos_addr)
            .namespace(namespace)
            .service_name(service_name)
            .bind_addr(service_addr)?;
        if let Some(host) = service_host {
            builder = builder.service_host(host);
        }
        match (username, password) {
            (Some(u), Some(p)) => builder = builder.auth(u, p),
            (None, None) => {}
            _ => {
                return Err(Error::invalid_config(
                    "NACOS_USERNAME and NACOS_PASSWORD must be provided together",
                ));
            }
        }
        builder.build()
    }
}

/// Fluent builder for [`ServiceConfig`].
#[derive(Debug, Default, Clone)]
pub struct ServiceConfigBuilder {
    nacos_addr: Option<String>,
    namespace: Option<String>,
    service_name: Option<String>,
    group: Option<String>,
    service_host: Option<String>,
    service_port: Option<u16>,
    weight: Option<f64>,
    ephemeral: Option<bool>,
    auth: Option<(String, String)>,
    metadata: HashMap<String, String>,
}

impl ServiceConfigBuilder {
    /// Set the Nacos server address (`host:port`).
    pub fn nacos_addr(mut self, addr: impl Into<String>) -> Self {
        self.nacos_addr = Some(addr.into());
        self
    }

    /// Set the namespace id.
    pub fn namespace(mut self, ns: impl Into<String>) -> Self {
        self.namespace = Some(ns.into());
        self
    }

    /// Set the service name.
    pub fn service_name(mut self, name: impl Into<String>) -> Self {
        self.service_name = Some(name.into());
        self
    }

    /// Set the service group (defaults to [`DEFAULT_GROUP`]).
    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }

    /// Set the advertised host registered to Nacos (defaults to the local IP).
    pub fn service_host(mut self, host: impl Into<String>) -> Self {
        self.service_host = Some(host.into());
        self
    }

    /// Set the advertised port registered to Nacos.
    pub fn service_port(mut self, port: u16) -> Self {
        self.service_port = Some(port);
        self
    }

    /// Parse the port out of a `host:port` string; the host portion is **not** used (the
    /// advertised host comes from [`service_host`](Self::service_host) or the local IP).
    ///
    /// Provided for compatibility with the `SERVICE_ADDR` environment variable convention.
    pub fn bind_addr(mut self, addr: impl AsRef<str>) -> Result<Self> {
        let addr = addr.as_ref();
        let (host, port) = addr.rsplit_once(':').ok_or_else(|| {
            Error::invalid_config(format!("invalid bind address `{addr}` (expect host:port)"))
        })?;
        if host.is_empty() {
            return Err(Error::invalid_config(format!(
                "invalid bind address `{addr}`: empty host"
            )));
        }
        let port: u16 = port.parse().map_err(|_| {
            Error::invalid_config(format!("invalid bind address `{addr}`: bad port"))
        })?;
        self.service_port = Some(port);
        Ok(self)
    }

    /// Set the instance weight.
    pub fn weight(mut self, weight: f64) -> Self {
        self.weight = Some(weight);
        self
    }

    /// Set whether the instance is ephemeral (defaults to `true`).
    pub fn ephemeral(mut self, ephemeral: bool) -> Self {
        self.ephemeral = Some(ephemeral);
        self
    }

    /// Set the auth credentials.
    pub fn auth(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.auth = Some((username.into(), password.into()));
        self
    }

    /// Insert a single metadata entry.
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Insert multiple metadata entries at once.
    pub fn metadata_all<I, K, V>(mut self, entries: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.metadata
            .extend(entries.into_iter().map(|(k, v)| (k.into(), v.into())));
        self
    }

    /// Validate and build the [`ServiceConfig`].
    pub fn build(self) -> Result<ServiceConfig> {
        let nacos_addr = require(self.nacos_addr, "nacos_addr")?;
        validate_host_port(&nacos_addr, "nacos_addr")?;
        let namespace = require(self.namespace, "namespace")?;
        let service_name = require(self.service_name, "service_name")?;
        let service_port = require(self.service_port, "service_port")?;
        let service_host = match self.service_host {
            Some(h) => h,
            None => local_ip_address::local_ip()?.to_string(),
        };
        let mut metadata = self.metadata;
        metadata
            .entry(META_GRPC_PORT.to_string())
            .or_insert_with(|| service_port.to_string());

        Ok(ServiceConfig {
            nacos_addr,
            namespace,
            service_name,
            group: self.group.unwrap_or_else(|| DEFAULT_GROUP.to_string()),
            service_host,
            service_port,
            weight: self.weight.unwrap_or(DEFAULT_WEIGHT),
            ephemeral: self.ephemeral.unwrap_or(true),
            auth: self.auth,
            metadata,
        })
    }
}

/// Read an environment variable, turning a miss into [`Error::Env`].
fn read_env(name: &str) -> Result<String> {
    env::var(name).map_err(|source| Error::Env {
        name: name.to_string(),
        source,
    })
}

/// Ensure a required builder field is present.
fn require<T>(value: Option<T>, field: &str) -> Result<T> {
    value.ok_or_else(|| Error::invalid_config(format!("missing required field `{field}`")))
}

/// Validate that `addr` looks like `host:port`, where `host` may be a hostname/IP and `port`
/// fits in a `u16`.
fn validate_host_port(addr: &str, field: &str) -> Result<()> {
    let (host, port) = addr.rsplit_once(':').ok_or_else(|| {
        Error::invalid_config(format!("invalid `{field}` = `{addr}` (expect host:port)"))
    })?;
    if host.is_empty() {
        return Err(Error::invalid_config(format!(
            "invalid `{field}` = `{addr}`: empty host"
        )));
    }
    port.parse::<u16>()
        .map_err(|_| Error::invalid_config(format!("invalid `{field}` = `{addr}`: bad port")))?;
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn builder_requires_mandatory_fields() {
        let err = ServiceConfig::builder().build().unwrap_err();
        assert!(matches!(err, Error::InvalidConfig(_)));
    }

    #[test]
    fn builder_bind_addr_extracts_port() {
        let cfg = ServiceConfig::builder()
            .nacos_addr("127.0.0.1:8848")
            .namespace("public")
            .service_name("svc")
            .bind_addr("0.0.0.0:9000")
            .unwrap()
            .service_host("10.0.0.1")
            .build()
            .unwrap();
        assert_eq!(cfg.service_port, 9000);
        assert_eq!(cfg.service_host, "10.0.0.1");
        assert_eq!(
            cfg.metadata.get(META_GRPC_PORT).map(String::as_str),
            Some("9000")
        );
    }

    #[test]
    fn builder_rejects_bad_bind_addr() {
        let err = ServiceConfig::builder()
            .bind_addr("no-port-here")
            .unwrap_err();
        assert!(matches!(err, Error::InvalidConfig(_)));

        let err = ServiceConfig::builder().bind_addr(":9000").unwrap_err();
        assert!(matches!(err, Error::InvalidConfig(_)));

        let err = ServiceConfig::builder()
            .bind_addr("host:not-a-port")
            .unwrap_err();
        assert!(matches!(err, Error::InvalidConfig(_)));
    }

    #[test]
    fn validate_host_port_accepts_hostname() {
        assert!(validate_host_port("nacos.internal:8848", "nacos_addr").is_ok());
        assert!(validate_host_port("localhost:8848", "nacos_addr").is_ok());
        assert!(validate_host_port("127.0.0.1:8848", "nacos_addr").is_ok());
    }

    #[test]
    fn validate_host_port_rejects_empty_host() {
        assert!(validate_host_port(":8848", "nacos_addr").is_err());
    }

    #[test]
    fn metadata_user_override_takes_precedence() {
        let cfg = ServiceConfig::builder()
            .nacos_addr("127.0.0.1:8848")
            .namespace("public")
            .service_name("svc")
            .service_host("1.2.3.4")
            .service_port(9000)
            .metadata(META_GRPC_PORT, "custom")
            .build()
            .unwrap();
        assert_eq!(
            cfg.metadata.get(META_GRPC_PORT).map(String::as_str),
            Some("custom")
        );
    }
}
