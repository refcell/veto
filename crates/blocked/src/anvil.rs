use crate::methods::{ANVIL_METHODS, blocked_method_set};
use http::Uri;
use std::net::SocketAddr;
use veto_config::Config;

/// Preconfigured Veto instance that blocks all documented Anvil-specific RPC methods.
#[derive(Debug, Clone)]
pub struct AnvilBlocked {
    bind_address: SocketAddr,
    upstream_url: Uri,
}

impl AnvilBlocked {
    /// Create a new preset targeting the given upstream node.
    pub const fn new(bind_address: SocketAddr, upstream_url: Uri) -> Self {
        Self {
            bind_address,
            upstream_url,
        }
    }

    /// Socket address Veto should bind to.
    pub const fn bind_address(&self) -> SocketAddr {
        self.bind_address
    }

    /// Upstream Anvil endpoint URL.
    pub const fn upstream_url(&self) -> &Uri {
        &self.upstream_url
    }

    /// All Anvil methods blocked by this preset.
    pub const fn methods() -> &'static [&'static str] {
        ANVIL_METHODS
    }

    /// Convert the preset into a [`Config`].
    pub fn into_config(self) -> Config {
        Config::new(self.bind_address, self.upstream_url, blocked_method_set())
    }

    /// Borrowed variant of [`into_config`](Self::into_config).
    pub fn to_config(&self) -> Config {
        Config::new(
            self.bind_address,
            self.upstream_url.clone(),
            blocked_method_set(),
        )
    }
}

impl From<AnvilBlocked> for Config {
    fn from(value: AnvilBlocked) -> Self {
        value.into_config()
    }
}
