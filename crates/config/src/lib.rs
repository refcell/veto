//! Configuration loading and resolution logic for the veto proxy.

use http::Uri;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use thiserror::Error;

/// Default socket address for the proxy to bind to.
pub const DEFAULT_BIND_ADDRESS: &str = "0.0.0.0:8546";

/// Default upstream JSON-RPC endpoint (Anvil).
pub const DEFAULT_UPSTREAM_URL: &str = "http://127.0.0.1:8545";

/// Default path for the configuration file.
pub const DEFAULT_CONFIG_PATH: &str = ".veto.toml";

/// Representation of the TOML configuration file.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct FileConfig {
    /// Address the proxy should bind to.
    pub bind_address: Option<String>,
    /// Upstream Anvil endpoint.
    pub upstream_url: Option<String>,
    /// Methods to block when encountered in JSON-RPC payloads.
    pub blocked_methods: Option<Vec<String>>,
}

/// Overrides provided via the CLI.
#[derive(Debug, Clone, Default)]
pub struct Overrides {
    bind_address: Option<SocketAddr>,
    upstream_url: Option<Uri>,
    blocked_methods: Vec<String>,
}

impl Overrides {
    /// Create a new overrides instance.
    pub fn new(
        bind_address: Option<SocketAddr>,
        upstream_url: Option<Uri>,
        blocked_methods: Vec<String>,
    ) -> Self {
        Self {
            bind_address,
            upstream_url,
            blocked_methods,
        }
    }

    /// Returns `true` if no overriding values were provided.
    pub fn is_empty(&self) -> bool {
        self.bind_address.is_none()
            && self.upstream_url.is_none()
            && self.blocked_methods.is_empty()
    }

    /// Accessor for the bind address override.
    pub fn bind_address(&self) -> Option<SocketAddr> {
        self.bind_address
    }

    /// Accessor for the upstream URL override.
    pub fn upstream_url(&self) -> Option<&Uri> {
        self.upstream_url.as_ref()
    }

    /// Borrow the blocked methods override.
    pub fn blocked_methods(&self) -> &[String] {
        &self.blocked_methods
    }
}

/// Fully resolved proxy configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    bind_address: SocketAddr,
    upstream_url: Uri,
    blocked_methods: HashSet<String>,
}

impl Config {
    /// Construct a new [`Config`].
    pub fn new(
        bind_address: SocketAddr,
        upstream_url: Uri,
        blocked_methods: HashSet<String>,
    ) -> Self {
        Self {
            bind_address,
            upstream_url,
            blocked_methods,
        }
    }

    /// Address the proxy server will bind to.
    pub fn bind_address(&self) -> SocketAddr {
        self.bind_address
    }

    /// Upstream JSON-RPC endpoint.
    pub fn upstream_url(&self) -> &Uri {
        &self.upstream_url
    }

    /// Blocked JSON-RPC method names (lowercase).
    pub fn blocked_methods(&self) -> &HashSet<String> {
        &self.blocked_methods
    }
}

/// Parse and load the configuration file if it exists.
pub fn load_file(path: &Path) -> Result<Option<FileConfig>, ConfigError> {
    if !path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let parsed: FileConfig =
        toml::from_str(&contents).map_err(|source| ConfigError::TomlParse { source })?;
    Ok(Some(parsed))
}

/// Resolve the final configuration by merging file values with CLI overrides.
pub fn resolve_config(
    file: Option<FileConfig>,
    overrides: Overrides,
) -> Result<Config, ConfigError> {
    let (file_bind, file_upstream, file_methods) = match file {
        Some(cfg) => (
            cfg.bind_address,
            cfg.upstream_url,
            cfg.blocked_methods.unwrap_or_default(),
        ),
        None => (None, None, Vec::new()),
    };

    let bind_address = if let Some(addr) = overrides.bind_address() {
        addr
    } else if let Some(value) = file_bind.as_deref() {
        parse_socket_addr(value)?
    } else {
        parse_socket_addr(DEFAULT_BIND_ADDRESS)?
    };

    let upstream_url = if let Some(uri) = overrides.upstream_url().cloned() {
        uri
    } else if let Some(value) = file_upstream.as_deref() {
        parse_uri(value)?
    } else {
        parse_uri(DEFAULT_UPSTREAM_URL)?
    };

    let mut blocked_methods: HashSet<String> = HashSet::new();
    blocked_methods.extend(
        file_methods
            .into_iter()
            .filter_map(|method| normalize_method(&method)),
    );
    blocked_methods.extend(
        overrides
            .blocked_methods()
            .iter()
            .filter_map(|method| normalize_method(method)),
    );

    Ok(Config::new(bind_address, upstream_url, blocked_methods))
}

fn parse_socket_addr(value: &str) -> Result<SocketAddr, ConfigError> {
    SocketAddr::from_str(value).map_err(|source| ConfigError::BindAddress {
        value: value.to_string(),
        source,
    })
}

fn parse_uri(value: &str) -> Result<Uri, ConfigError> {
    Uri::from_str(value).map_err(|source| ConfigError::UpstreamUrl {
        value: value.to_string(),
        source,
    })
}

fn normalize_method(value: &str) -> Option<String> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

/// Errors that can occur while loading the configuration.
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Underlying IO failure.
    #[error("failed to read config file {path:?}: {source}")]
    Io {
        path: std::path::PathBuf,
        source: std::io::Error,
    },
    /// Failure to parse TOML.
    #[error("unable to parse config file as TOML: {source}")]
    TomlParse {
        #[from]
        source: toml::de::Error,
    },
    /// Invalid socket address for bind.
    #[error("invalid bind address '{value}': {source}")]
    BindAddress {
        value: String,
        source: std::net::AddrParseError,
    },
    /// Invalid upstream URL.
    #[error("invalid upstream url '{value}': {source}")]
    UpstreamUrl { value: String, source: http::Error },
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn default_resolution_without_inputs() {
        let config = resolve_config(None, Overrides::default()).expect("config resolves");
        assert_eq!(
            config.bind_address(),
            parse_socket_addr(DEFAULT_BIND_ADDRESS).unwrap()
        );
        assert_eq!(
            config.upstream_url(),
            &parse_uri(DEFAULT_UPSTREAM_URL).unwrap()
        );
        assert!(config.blocked_methods().is_empty());
    }

    #[rstest]
    fn file_values_are_used() {
        let file = FileConfig {
            bind_address: Some("127.0.0.1:9000".to_string()),
            upstream_url: Some("http://127.0.0.1:9001".to_string()),
            blocked_methods: Some(vec!["eth_sendTransaction".into(), " personal_sign ".into()]),
        };
        let config = resolve_config(Some(file), Overrides::default()).expect("config resolves");
        assert_eq!(
            config.bind_address(),
            "127.0.0.1:9000".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(
            config.upstream_url(),
            &"http://127.0.0.1:9001".parse::<Uri>().unwrap()
        );
        let methods = config.blocked_methods();
        assert!(methods.contains("eth_sendtransaction"));
        assert!(methods.contains("personal_sign"));
    }

    #[rstest]
    fn cli_overrides_take_precedence() {
        let file = FileConfig {
            bind_address: Some("127.0.0.1:9000".to_string()),
            upstream_url: Some("http://127.0.0.1:9001".to_string()),
            blocked_methods: Some(vec!["eth_sendtransaction".into()]),
        };

        let overrides = Overrides::new(
            Some("127.0.0.1:9100".parse().unwrap()),
            Some("http://127.0.0.1:9101".parse().unwrap()),
            vec!["eth_getBalance".into()],
        );

        let config = resolve_config(Some(file), overrides).expect("config resolves");

        assert_eq!(
            config.bind_address(),
            "127.0.0.1:9100".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(
            config.upstream_url(),
            &"http://127.0.0.1:9101".parse::<Uri>().unwrap()
        );
        assert!(config.blocked_methods().contains("eth_sendtransaction"));
        assert!(config.blocked_methods().contains("eth_getbalance"));
    }

    #[rstest]
    fn invalid_bind_address_yields_error() {
        let file = FileConfig {
            bind_address: Some("not-an-addr".into()),
            ..Default::default()
        };

        let err = resolve_config(Some(file), Overrides::default()).unwrap_err();
        match err {
            ConfigError::BindAddress { value, .. } => assert_eq!(value, "not-an-addr"),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[rstest]
    fn normalize_method_discards_empty_entries() {
        let file = FileConfig {
            bind_address: None,
            upstream_url: None,
            blocked_methods: Some(vec!["  ".into(), "eth_call".into()]),
        };

        let config = resolve_config(Some(file), Overrides::default()).expect("config resolves");
        assert_eq!(config.blocked_methods().len(), 1);
        assert!(config.blocked_methods().contains("eth_call"));
    }
}
