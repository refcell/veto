#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/refcell/veto/main/assets/square.png",
    html_favicon_url = "https://raw.githubusercontent.com/refcell/veto/main/assets/favicon.ico",
    issue_tracker_base_url = "https://github.com/refcell/veto/issues/"
)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

mod config;
mod constants;
mod errors;
mod file;
mod overrides;
mod resolver;

pub use config::Config;
pub use constants::{DEFAULT_BIND_ADDRESS, DEFAULT_CONFIG_PATH, DEFAULT_UPSTREAM_URL};
pub use errors::ConfigError;
pub use file::{FileConfig, load_file};
pub use overrides::Overrides;
pub use resolver::resolve_config;
