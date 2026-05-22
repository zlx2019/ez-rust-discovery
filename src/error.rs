use std::io;
use std::net::AddrParseError;
use std::num::ParseIntError;

/// Crate-wide `Result` alias.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors produced during service-discovery operations.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// I/O error, typically from the underlying network or runtime setup.
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    /// Failed to read an environment variable.
    #[error("missing or invalid environment variable `{name}`: {source}")]
    Env {
        /// Name of the environment variable.
        name: String,
        /// Underlying cause.
        #[source]
        source: std::env::VarError,
    },

    /// Failed to parse a socket address.
    #[error("invalid socket address: {0}")]
    AddrParse(#[from] AddrParseError),

    /// Failed to parse a port number.
    #[error("invalid port number: {0}")]
    PortParse(#[from] ParseIntError),

    /// Failed to detect the local IP address.
    #[error("failed to detect local IP: {0}")]
    LocalIp(#[from] local_ip_address::Error),

    /// Error raised by the Nacos client during build or registration.
    #[error("nacos error: {0}")]
    Nacos(#[from] nacos_sdk::api::error::Error),

    /// Invalid configuration (missing required fields, malformed address, etc.).
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),
}

impl Error {
    /// Build an [`Error::InvalidConfig`].
    pub(crate) fn invalid_config(msg: impl Into<String>) -> Self {
        Self::InvalidConfig(msg.into())
    }
}
