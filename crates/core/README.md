# `veto-core`

<p align="center">
  <a href="https://github.com/refcell/veto/actions/workflows/rust_ci.yaml"><img src="https://github.com/refcell/veto/actions/workflows/rust_ci.yaml/badge.svg?label=ci" alt="CI"></a>
  <a href="https://github.com/refcell/veto/actions/workflows/examples.yaml"><img src="https://github.com/refcell/veto/actions/workflows/examples.yaml/badge.svg?label=examples" alt="Examples"></a>
  <img src="https://img.shields.io/badge/Rust-1.88+-orange.svg?labelColor=2a2f35" alt="Rust">
  <a href="https://github.com/refcell/veto/pkgs/container/veto%2Fveto-builder"><img src="https://img.shields.io/badge/docker-ghcr.io-blue?logo=docker&logoColor=white" alt="Docker"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-2ea44f.svg?labelColor=2a2f35" alt="License"></a>
</p>

Core runtime for the Veto JSON-RPC proxy.

## Features

- **Runtime** – [`run`] bootstraps the Axum server, binds to the requested socket, and awaits Ctrl+C for shutdown
- **Proxy Engine** – [`router`] wires handlers around [`AppState`] so the proxy can forward or block JSON-RPC calls
- **JSON-RPC Validation** – strict parsing guards against malformed payloads and rejects batch requests up front
- **Error Reporting** – deterministic error payloads and rich [`ProxyError`] diagnostics for callers

## Usage

```rust,no_run
use veto_config::{resolve_config, Overrides};
use veto_core::run;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = resolve_config(None, Overrides::default())?;
    run(config).await?;
    Ok(())
}
```

## Contributing

Pull requests are welcome! [Open an issue](https://github.com/refcell/veto/issues/new) to discuss ideas or report bugs before sending a patch.
