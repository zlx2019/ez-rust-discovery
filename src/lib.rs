use std::{env, io::Error};
use std::collections::HashMap;
use std::env::VarError;
use std::net::{AddrParseError, SocketAddr};
use std::sync::Arc;
use futures::executor::block_on;
use futures::TryFutureExt;
use local_ip_address::local_ip;
use nacos_sdk::api::constants;
use nacos_sdk::api::naming::{NamingService, NamingServiceBuilder, ServiceInstance};
use nacos_sdk::api::props::ClientProps;
use tracing::info;

const META_GRPC_PORT: &'static str = "gRPC_port";

#[derive(Debug)]
pub enum EzError {
    IO(Error),
    Env(VarError, String),
    Parse(AddrParseError),
    LocalIP(local_ip_address::Error),
    Other(String)
}

impl std::fmt::Display for EzError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EzError::IO(err) => write!(f,"IO Error: {}", err),
            EzError::Env(err, name) => write!(f,"Read environment variables [{}] error: {}", name, err),
            EzError::Parse(err) => write!(f,"Parse error: {}", err),
            EzError::LocalIP(err) => write!(f,"Local IP error: {}", err),
            EzError::Other(msg) => write!(f, "Other error: {}", msg),
        }
    }
}

impl std::error::Error for EzError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            EzError::IO(err) => Some(err),
            EzError::Env(err, _) => Some(err),
            EzError::Parse(err) => Some(err),
            EzError::LocalIP(err) => Some(err),
            _ => None,
        }
    }
}

impl From<Error> for EzError {
    fn from(value: Error) -> Self {
        EzError::IO(value)
    }
}

impl From<AddrParseError> for EzError {
    fn from(value: AddrParseError) -> Self {
        EzError::Parse(value)
    }
}

impl From<local_ip_address::Error> for EzError {
    fn from(value: local_ip_address::Error) -> Self {
        EzError::LocalIP(value)
    }
}

pub struct ServeOptions {
    pub addr: Option<String>,
    pub namespace: Option<String>,
    pub service_addr: Option<String>,
    pub service_name: Option<String>,
    pub service_host: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ServiceManager {
    pub naming_service: Arc<NamingService>,
    pub service_instance: ServiceInstance,
    pub service_name: String,
}

impl ServiceManager {
    pub fn new(opt: ServeOptions) -> Result<Self,EzError> {
        let addr = parse_addr(match opt.addr {
            Some(addr) => addr,
            None => get_env("NACOS_ADDR")?
        })?;
        let namespace = match opt.namespace {
            Some(namespace) => namespace,
            None => get_env("NACOS_NAMESPACE")?
        };
        let service_addr = parse_addr(match opt.service_addr {
            Some(service_addr) => parse_addr(service_addr)?,
            None => get_env("SERVICE_ADDR")?
        })?;
        let service_name = match opt.service_name {
            Some(service_name) => service_name,
            None => get_env("SERVICE_NAME")?
        };
        let local_ip = local_ip()?.to_string();
        let service_host = opt.service_host.unwrap_or_else(|| get_env("SERVICE_HOST").unwrap_or(local_ip));
        info!("[NACOS_ADDR]: {}", addr);
        info!("[NACOS_NAMESPACE]: {}", namespace);
        info!("[SERVICE_ADDR]: {}", service_addr);
        info!("[SERVICE_NAME]: {}", service_name);
        info!("[SERVICE_HOST]: {}", service_host);
        let naming_service = NamingServiceBuilder::new(
            ClientProps::new().server_addr(addr).namespace(namespace))
            .build()
            .map_err(|e| {
                EzError::Other(format!("NamingService create failed: {}", e))
            })?;
        let (_, port) = match service_addr.split_once(":") {
            Some(value) => value,
            None => return Err(EzError::Other("Invalid service address".to_string()))
        };


        let instance = ServiceInstance{
            ip: service_host,
            port: port.parse::<i32>().unwrap(),
            weight: 1.0,
            healthy: true,
            enabled: true,
            ephemeral: true,
            metadata: HashMap::from([(META_GRPC_PORT.to_string(), port.to_string())]),
            ..Default::default()
        };
        Ok(Self{
            naming_service: Arc::new(naming_service),
            service_instance: instance,
            service_name,
        })
    }

    pub fn online(&self) -> Result<(),EzError> {
        block_on(self.naming_service.register_instance(self.service_name.clone(), Some(constants::DEFAULT_GROUP.to_string()), self.service_instance.clone()))
            .map_err(|e| EzError::Other(format!("Service online error: {}", e.to_string())))?;
        info!("Service online successfully");
        Ok(())
    }
    pub fn offline(&self) -> Result<(), EzError> {
        block_on(self.naming_service.deregister_instance(self.service_name.clone(), Some(constants::DEFAULT_GROUP.to_string()), self.service_instance.clone())
            .map_err(|e| EzError::Other(format!("Service offline error: {}", e.to_string()))))?;
        info!("Service offline successfully");
        Ok(())
    }
}

impl Default for ServeOptions {
    fn default() -> Self {
        Self{
            addr: None,
            namespace: None,
            service_addr: None,
            service_name: None,
            service_host: None,
        }
    }
}

fn get_env(name: &str) -> Result<String, EzError> {
    let res = env::var(name);
    match res {
        Ok(val) => Ok(val),
        Err(err) => Err(EzError::Env(err, String::from(name)))
    }
}

fn parse_addr(addr: String) -> Result<String, EzError> {
    match addr.parse::<SocketAddr>() {
        Ok(_) => Ok(addr),
        Err(err) => Err(EzError::Parse(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    #[test]
    fn test_online(){
        let manager = ServiceManager::new(ServeOptions::default()).unwrap();
        manager.online().unwrap();
        sleep(std::time::Duration::from_secs(10));
        manager.offline().unwrap();
    }
}
