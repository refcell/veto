//! Demonstrates how to run `veto` in front of an Anvil node and block every
//! custom `anvil_` JSON-RPC method.
//!
//! The example spins up:
//! 1. A disposable Anvil instance listening on a random localhost port.
//! 2. A `veto` proxy that forwards traffic to Anvil while blocking the custom methods.
//! 3. A simple JSON-RPC client that tries each blocked method via the proxy and
//!    prints the resulting error message.
//!
//! The list of methods is sourced from the Anvil documentation:
//! <https://getfoundry.sh/anvil/reference#custom-methods>.
//! Running this example requires Anvil to be installed and available on `$PATH`
//! (e.g. via `foundryup`).

use anyhow::{Context, Result, anyhow};
use http::Uri;
use reqwest::Client;
use serde_json::{Value, json};
use std::collections::HashSet;
use std::env;
use std::net::{SocketAddr, TcpListener};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::sleep;
use veto_config::Config;
use veto_core::run;

/// All documented Anvil custom RPC methods.
const CUSTOM_ANVIL_METHODS: &[&str] = &[
    "anvil_autoImpersonateAccount",
    "anvil_dropTransaction",
    "anvil_dumpState",
    "anvil_enableTraces",
    "anvil_getAutomine",
    "anvil_impersonateAccount",
    "anvil_increaseTime",
    "anvil_loadState",
    "anvil_metadata",
    "anvil_mine",
    "anvil_mine_detailed",
    "anvil_nodeInfo",
    "anvil_removeBlockTimestampInterval",
    "anvil_reset",
    "anvil_revert",
    "anvil_setAutomine",
    "anvil_setBalance",
    "anvil_setBlockGasLimit",
    "anvil_setBlockTimestampInterval",
    "anvil_setChainId",
    "anvil_setCode",
    "anvil_setCoinbase",
    "anvil_setIntervalMining",
    "anvil_setLoggingEnabled",
    "anvil_setMinGasPrice",
    "anvil_setNextBlockBaseFeePerGas",
    "anvil_setNextBlockTimestamp",
    "anvil_setNonce",
    "anvil_setRpcUrl",
    "anvil_setStorageAt",
    "anvil_setTime",
    "anvil_snapshot",
    "anvil_stopImpersonatingAccount",
];

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let anvil_port = reserve_port("Anvil")?;
    let proxy_port = reserve_port("veto proxy")?;

    let mut anvil = spawn_anvil(anvil_port)?;
    wait_for_port(SocketAddr::from(([127, 0, 0, 1], anvil_port)), "Anvil").await?;

    let proxy_addr = SocketAddr::from(([127, 0, 0, 1], proxy_port));
    let upstream: Uri = format!("http://127.0.0.1:{anvil_port}").parse().unwrap();
    let config = Config::new(proxy_addr, upstream, blocked_methods());

    let proxy_task = tokio::spawn(async move {
        if let Err(error) = run(config).await {
            eprintln!("veto proxy exited with error: {error}");
        }
    });
    wait_for_port(proxy_addr, "veto proxy").await?;

    println!("Proxy ready at http://{proxy_addr}");
    println!("Blocked methods:");
    for method in CUSTOM_ANVIL_METHODS {
        println!("  - {method}");
    }
    println!();

    let blocked = exercise_methods(proxy_addr).await?;

    println!("Blocked responses:");
    for (method, message) in blocked {
        println!("{method}: {message}");
    }

    proxy_task.abort();
    // Ignore errors: the task is aborted because the example is done.
    let _ = proxy_task.await;
    terminate(&mut anvil, "Anvil")?;

    Ok(())
}

fn blocked_methods() -> HashSet<String> {
    CUSTOM_ANVIL_METHODS
        .iter()
        .map(|method| method.to_ascii_lowercase())
        .collect()
}

fn reserve_port(reason: &str) -> Result<u16> {
    let listener = TcpListener::bind("127.0.0.1:0")
        .with_context(|| format!("failed to reserve a port for {reason}"))?;
    let port = listener
        .local_addr()
        .with_context(|| format!("failed to read local address for {reason} listener"))?
        .port();
    drop(listener);
    Ok(port)
}

fn spawn_anvil(port: u16) -> Result<Child> {
    let binary = env::var("ANVIL_BIN").unwrap_or_else(|_| "anvil".into());
    Command::new(binary)
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(port.to_string())
        .arg("--silent")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("failed to launch anvil")
}

async fn wait_for_port(address: SocketAddr, label: &str) -> Result<()> {
    for _ in 0..50 {
        if TcpStream::connect(address).await.is_ok() {
            return Ok(());
        }
        sleep(Duration::from_millis(100)).await;
    }

    Err(anyhow!("timed out waiting for {label} on {address}"))
}

async fn exercise_methods(address: SocketAddr) -> Result<Vec<(String, String)>> {
    let client = Client::new();
    let url = format!("http://{address}");

    let mut results = Vec::with_capacity(CUSTOM_ANVIL_METHODS.len());
    for method in CUSTOM_ANVIL_METHODS {
        let payload = json!({
            "jsonrpc": "2.0",
            "id": method,
            "method": method,
            "params": []
        });
        let response = client
            .post(&url)
            .header("content-type", "application/json")
            .body(payload.to_string())
            .send()
            .await
            .with_context(|| format!("proxy call for {method} failed"))?;
        let value: Value = response
            .json()
            .await
            .with_context(|| format!("response from proxy was not JSON for {method}"))?;

        let message = value["error"]["message"]
            .as_str()
            .unwrap_or("unexpected response")
            .to_string();

        results.push((method.to_string(), message));
    }

    Ok(results)
}

fn terminate(child: &mut Child, label: &str) -> Result<()> {
    match child.try_wait()? {
        Some(_) => Ok(()),
        None => {
            child
                .kill()
                .with_context(|| format!("failed to kill {label}"))?;
            child.wait().ok();
            Ok(())
        }
    }
}
