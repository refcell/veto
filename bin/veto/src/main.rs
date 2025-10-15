use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use http::Uri;
use tracing::{info, warn};
use veto_config::{DEFAULT_CONFIG_PATH, Overrides, load_file, resolve_config};

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

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();

    let file_config = match load_file(cli.config.as_path())? {
        Some(cfg) => Some(cfg),
        None => {
            if cli.config.as_path() != std::path::Path::new(DEFAULT_CONFIG_PATH) {
                warn!(
                    "configuration file {:?} not found; continuing with defaults and CLI overrides",
                    cli.config
                );
            }
            None
        }
    };

    let overrides = Overrides::new(cli.bind_address, cli.upstream_url, cli.blocked_methods);

    let config = resolve_config(file_config, overrides)?;

    info!(
        "starting proxy on http://{} forwarding to {}",
        config.bind_address(),
        config.upstream_url()
    );
    if !config.blocked_methods().is_empty() {
        let mut blocked: Vec<_> = config.blocked_methods().iter().cloned().collect();
        blocked.sort();
        info!("blocking methods: {}", blocked.join(", "));
    } else {
        info!("no blocked methods configured");
    }

    veto_core::run(config).await?;
    Ok(())
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .try_init();
}
