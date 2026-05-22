use std::future::Future;
use std::sync::Arc;

use nacos_sdk::api::naming::{NamingService, NamingServiceBuilder, ServiceInstance};
use nacos_sdk::api::props::ClientProps;
use tracing::{debug, info};

use crate::config::ServiceConfig;
use crate::error::{Error, Result};

/// Entry point for service registration and deregistration.
///
/// A [`ServiceManager`] owns a Nacos `NamingService` client together with a prepared
/// [`ServiceInstance`]; call [`register`](Self::register) /
/// [`deregister`](Self::deregister) to bring the instance up or down.
///
/// The type is `Clone` and shares its inner state through `Arc`, so it can be cheaply
/// passed across tasks.
#[derive(Clone)]
pub struct ServiceManager {
    inner: Arc<Inner>,
}

struct Inner {
    naming: NamingService,
    instance: ServiceInstance,
    service_name: String,
    group: String,
}

impl ServiceManager {
    /// Asynchronously construct a [`ServiceManager`] from a [`ServiceConfig`].
    ///
    /// This only initializes the underlying Nacos client; the instance is **not** registered
    /// until [`register`](Self::register) is called.
    pub async fn new(config: ServiceConfig) -> Result<Self> {
        debug!(
            nacos_addr = %config.nacos_addr,
            namespace = %config.namespace,
            service = %config.service_name,
            service_host = %config.service_host,
            service_port = config.service_port,
            auth = config.auth.is_some(),
            "creating ServiceManager"
        );

        let mut client_props = ClientProps::new()
            .server_addr(&config.nacos_addr)
            .namespace(&config.namespace);

        let auth_enabled = config.auth.is_some();
        if let Some((user, pass)) = config.auth.as_ref() {
            client_props = client_props.auth_username(user).auth_password(pass);
        }

        let mut builder = NamingServiceBuilder::new(client_props);
        if auth_enabled {
            // Enable the HTTP auth plugin; otherwise username/password would be ignored.
            builder = builder.enable_auth_plugin_http();
        }
        let naming = builder.build().await?;

        let instance = ServiceInstance {
            ip: config.service_host.clone(),
            port: i32::from(config.service_port),
            weight: config.weight,
            healthy: true,
            enabled: true,
            ephemeral: config.ephemeral,
            metadata: config.metadata.clone(),
            ..Default::default()
        };

        Ok(Self {
            inner: Arc::new(Inner {
                naming,
                instance,
                service_name: config.service_name,
                group: config.group,
            }),
        })
    }

    /// Synchronously construct a [`ServiceManager`] using a temporary tokio runtime.
    ///
    /// Intended for purely synchronous programs. From an async context, prefer
    /// [`new`](Self::new) instead.
    pub fn new_blocking(config: ServiceConfig) -> Result<Self> {
        block_on(Self::new(config))
    }

    /// Register the service instance with Nacos.
    pub async fn register(&self) -> Result<()> {
        self.inner
            .naming
            .register_instance(
                self.inner.service_name.clone(),
                Some(self.inner.group.clone()),
                self.inner.instance.clone(),
            )
            .await?;
        info!(
            service = %self.inner.service_name,
            group = %self.inner.group,
            host = %self.inner.instance.ip,
            port = self.inner.instance.port,
            "service registered"
        );
        Ok(())
    }

    /// Deregister the service instance from Nacos.
    pub async fn deregister(&self) -> Result<()> {
        self.inner
            .naming
            .deregister_instance(
                self.inner.service_name.clone(),
                Some(self.inner.group.clone()),
                self.inner.instance.clone(),
            )
            .await?;
        info!(
            service = %self.inner.service_name,
            group = %self.inner.group,
            "service deregistered"
        );
        Ok(())
    }

    /// Blocking variant of [`register`](Self::register).
    ///
    /// **Do not** call from within an existing tokio runtime — it will panic.
    pub fn register_blocking(&self) -> Result<()> {
        block_on(self.register())
    }

    /// Blocking variant of [`deregister`](Self::deregister).
    ///
    /// **Do not** call from within an existing tokio runtime — it will panic.
    pub fn deregister_blocking(&self) -> Result<()> {
        block_on(self.deregister())
    }

    /// Name of the registered service.
    pub fn service_name(&self) -> &str {
        &self.inner.service_name
    }

    /// Group of the registered service.
    pub fn group(&self) -> &str {
        &self.inner.group
    }

    /// Read-only view of the registered instance (host, port, metadata, ...).
    pub fn instance(&self) -> &ServiceInstance {
        &self.inner.instance
    }
}

impl std::fmt::Debug for ServiceManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServiceManager")
            .field("service_name", &self.inner.service_name)
            .field("group", &self.inner.group)
            .field("ip", &self.inner.instance.ip)
            .field("port", &self.inner.instance.port)
            .finish()
    }
}

/// Block on a future using an ad-hoc current-thread tokio runtime.
fn block_on<F, T>(fut: F) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(Error::Io)?;
    rt.block_on(fut)
}
