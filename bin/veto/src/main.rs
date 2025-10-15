#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/refcell/veto/main/assets/square.png",
    html_favicon_url = "https://raw.githubusercontent.com/refcell/veto/main/assets/favicon.ico",
    issue_tracker_base_url = "https://github.com/refcell/veto/issues/"
)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use anyhow::Result;

mod cli;
mod telemetry;

#[tokio::main]
async fn main() -> Result<()> {
    telemetry::init();
    cli::run().await
}
