# `veto-blocked`

Ready-to-use blocklists for the Veto JSON-RPC proxy.

## Presets

- **Anvil** – [`AnvilBlocked`] wires Veto in front of an Anvil node and blocks every `anvil_*` method the node exposes.

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
