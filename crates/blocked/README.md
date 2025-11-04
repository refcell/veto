# `veto-blocked`

<p align="center">
  <a href="https://github.com/refcell/veto/actions/workflows/rust_ci.yaml"><img src="https://github.com/refcell/veto/actions/workflows/rust_ci.yaml/badge.svg?label=ci" alt="CI"></a>
  <a href="https://github.com/refcell/veto/actions/workflows/examples.yaml"><img src="https://github.com/refcell/veto/actions/workflows/examples.yaml/badge.svg?label=examples" alt="Examples"></a>
  <img src="https://img.shields.io/badge/Rust-1.88+-orange.svg?labelColor=2a2f35" alt="Rust">
  <a href="https://github.com/refcell/veto/pkgs/container/veto%2Fveto-builder"><img src="https://img.shields.io/badge/docker-ghcr.io-blue?logo=docker&logoColor=white" alt="Docker"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-2ea44f.svg?labelColor=2a2f35" alt="License"></a>
</p>

Ready-to-use blocklists for the Veto JSON-RPC proxy.

## Presets

- **Anvil** – [`AnvilBlocked`] wires Veto in front of an Anvil node and blocks every `anvil_*` method the node exposes, alongside Hardhat/Ganache-style `evm_*` helpers.

## Usage

```rust
# use http::Uri;
# use std::net::SocketAddr;
# use veto_blocked::AnvilBlocked;
let bind = SocketAddr::from(([127, 0, 0, 1], 9000));
let upstream: Uri = "http://127.0.0.1:8545".parse()?;
let config = AnvilBlocked::new(bind, upstream).into_config();
# Ok::<_, http::uri::InvalidUriParts>(())
```

## Contributing

See something missing from a preset? [Open an issue](https://github.com/refcell/veto/issues/new) so we can keep the proxy secure by default.
