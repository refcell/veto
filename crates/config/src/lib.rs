#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/refcell/veto/main/assets/square.png",
    html_favicon_url = "https://raw.githubusercontent.com/refcell/veto/main/assets/favicon.ico",
    issue_tracker_base_url = "https://github.com/refcell/veto/issues/"
)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

mod config;
pub use config::Config;

mod constants;
pub use constants::{DEFAULT_BIND_ADDRESS, DEFAULT_CONFIG_PATH, DEFAULT_UPSTREAM_URL};

mod errors;
pub use errors::ConfigError;

mod file;
pub use file::{FileConfig, load_file};

mod overrides;
pub use overrides::Overrides;

mod resolver;
pub use resolver::resolve_config;
