# `veto-config`

<p align="center">
  <a href="https://github.com/refcell/veto/actions/workflows/rust_ci.yaml"><img src="https://github.com/refcell/veto/actions/workflows/rust_ci.yaml/badge.svg?label=ci" alt="CI"></a>
  <a href="https://github.com/refcell/veto/actions/workflows/examples.yaml"><img src="https://github.com/refcell/veto/actions/workflows/examples.yaml/badge.svg?label=examples" alt="Examples"></a>
  <img src="https://img.shields.io/badge/Rust-1.88+-orange.svg?labelColor=2a2f35" alt="Rust">
  <a href="https://github.com/refcell/veto/pkgs/container/veto%2Fveto-builder"><img src="https://img.shields.io/badge/docker-ghcr.io-blue?logo=docker&logoColor=white" alt="Docker"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-2ea44f.svg?labelColor=2a2f35" alt="License"></a>
</p>

Configuration management for the Veto JSON-RPC proxy.

## Features

- **Configuration File** – [`FileConfig`] mirrors the on-disk `.veto.toml`
- **CLI Overrides** – [`Overrides`] captures runtime flags and environment tweaks
- **Resolution Pipeline** – [`resolve_config`] merges defaults, files, and overrides into a [`Config`]
- **Defaults** – [`DEFAULT_BIND_ADDRESS`], [`DEFAULT_UPSTREAM_URL`], [`DEFAULT_CONFIG_PATH`] centralize proxy constants
- **Normalization** – method names are trimmed, lowercased, and deduplicated before reaching the runtime

## Usage

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use std::path::Path;
use veto_config::{load_file, resolve_config, Overrides};

let file = load_file(Path::new(".veto.toml"))?;
let overrides = Overrides::default();
let config = resolve_config(file, overrides)?;

println!("Proxy listening on {}", config.bind_address());
println!("Forwarding to {}", config.upstream_url());
# Ok(())
# }
```

## Contributing

Found something missing? [Open an issue](https://github.com/refcell/veto/issues/new) or send a PR so we can improve the proxy together.
