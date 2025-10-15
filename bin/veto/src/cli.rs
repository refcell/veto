//! Contains the CLI logic for the veto application.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Parser;
use http::Uri;
use tracing::{info, warn};
use veto_config::{Config, DEFAULT_CONFIG_PATH, FileConfig, Overrides, load_file, resolve_config};

/// Execute the veto CLI workflow.
pub(crate) async fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = cli.resolve_configuration()?;

    log_configuration(&config);

    veto_core::run(config).await?;
    Ok(())
}

fn log_configuration(config: &Config) {
    info!(
        "starting proxy on http://{} forwarding to {}",
        config.bind_address(),
        config.upstream_url()
    );

    if config.blocked_methods().is_empty() {
        info!("no blocked methods configured");
    } else {
        let mut blocked: Vec<_> = config.blocked_methods().iter().cloned().collect();
        blocked.sort();
        info!("blocking methods: {}", blocked.join(", "));
    }
}

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    about = "Ethereum JSON-RPC proxy with method filtering."
)]
struct Cli {
    /// Path to the TOML configuration file.
    #[arg(long, value_name = "PATH", default_value = DEFAULT_CONFIG_PATH)]
    config: PathBuf,

    /// Override the bind address for the proxy (e.g. 0.0.0.0:8546).
    #[arg(long, value_name = "ADDR")]
    bind_address: Option<SocketAddr>,

    /// Override the upstream Anvil endpoint (e.g. http://127.0.0.1:8545).
    #[arg(long, value_name = "URL")]
    upstream_url: Option<Uri>,

    /// Comma separated JSON-RPC method names to block.
    #[arg(long = "blocked-methods", value_delimiter = ',', value_name = "METHOD")]
    blocked_methods: Vec<String>,
}

impl Cli {
    fn resolve_configuration(&self) -> Result<Config> {
        let file_config = self.load_file_configuration()?;
        let overrides = Overrides::new(
            self.bind_address,
            self.upstream_url.clone(),
            self.blocked_methods.clone(),
        );

        Ok(resolve_config(file_config, overrides)?)
    }

    fn load_file_configuration(&self) -> Result<Option<FileConfig>> {
        let file_config = load_file(self.config.as_path())?;

        if file_config.is_none() && self.config.as_path() != Path::new(DEFAULT_CONFIG_PATH) {
            warn!(
                "configuration file {:?} not found; continuing with defaults and CLI overrides",
                self.config
            );
        }

        Ok(file_config)
    }
}
