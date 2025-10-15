# `veto-core`

Core runtime for the Veto JSON-RPC proxy.

## Features

- **Runtime** – [`run`] bootstraps the Axum server with graceful shutdown handling
- **Proxy Engine** – internal router forwards requests to the configured upstream while honoring blocklists
- **JSON-RPC Validation** – strict parsing guards against malformed or batch requests before forwarding
- **Error Reporting** – consistent JSON-RPC error payloads and rich [`ProxyError`] diagnostics for callers

## Usage

```rust,no_run
use veto_config::{resolve_config, Overrides};
use veto_core::run;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = resolve_config(None, Overrides::default())?;
    run(config).await?;
    Ok(())
}
```

## Contributing

Pull requests are welcome! [Open an issue](https://github.com/refcell/veto/issues/new) to discuss ideas or report bugs before sending a patch.
