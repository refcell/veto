//! Core proxy implementation for veto.

use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use axum::routing::any;
use http::{HeaderMap, Uri};
use http_body_util::BodyExt;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::rt::TokioExecutor;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::HashSet;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::TcpListener;
use tracing::{error, info};
use veto_config::Config;

/// Entry point for running the proxy server until shutdown.
pub async fn run(config: Config) -> Result<(), ProxyError> {
    let state = AppState::try_from_config(config)?;
    let bind_address = state.bind_address;

    let router = Router::new().fallback(any(proxy_handler)).with_state(state);

    let listener = TcpListener::bind(bind_address)
        .await
        .map_err(ProxyError::Bind)?;
    info!("veto proxy listening on http://{bind_address}");

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(ProxyError::Server)
}

#[derive(Clone)]
struct AppState {
    bind_address: std::net::SocketAddr,
    upstream: Uri,
    blocked_methods: Arc<HashSet<String>>,
    client: Client<HttpConnector, Body>,
}

impl AppState {
    fn try_from_config(config: Config) -> Result<Self, ProxyError> {
        let mut connector = HttpConnector::new();
        connector.enforce_http(false);
        let client = Client::builder(TokioExecutor::new()).build(connector);

        Ok(Self {
            bind_address: config.bind_address(),
            upstream: config.upstream_url().clone(),
            blocked_methods: Arc::new(config.blocked_methods().clone()),
            client,
        })
    }
}

async fn proxy_handler(State(state): State<AppState>, req: Request<Body>) -> Response {
    match process_request(&state, req).await {
        Ok(response) => response,
        Err(HandlerError::JsonRpc(response)) => response,
        Err(HandlerError::Internal(error)) => {
            error!("unexpected proxy error: {error}");
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("internal server error"))
                .expect("valid response")
        }
    }
}

async fn process_request(state: &AppState, req: Request<Body>) -> Result<Response, HandlerError> {
    let (parts, body) = req.into_parts();
    let collected = body
        .collect()
        .await
        .map_err(|error| HandlerError::from(ProxyError::Body(Box::new(error))))?;
    let bytes = collected.to_bytes();

    let json_rpc = parse_json_rpc(&bytes).map_err(HandlerError::from)?;
    let normalized_method = json_rpc.method.to_ascii_lowercase();

    if state.blocked_methods.contains(&normalized_method) {
        let error_payload = blocked_method_response(&json_rpc.id, &json_rpc.method);
        return Ok(error_payload);
    }

    let target_uri = build_target_uri(&state.upstream, &parts.uri).map_err(HandlerError::from)?;

    let mut forward_parts = parts;
    forward_parts.uri = target_uri;

    let mut forward_request = Request::from_parts(forward_parts, Body::from(bytes));
    sanitize_request_headers(forward_request.headers_mut());

    let response = state
        .client
        .request(forward_request)
        .await
        .map_err(|error| HandlerError::from(ProxyError::Upstream(error)))?;

    Ok(response)
}

fn sanitize_request_headers(headers: &mut HeaderMap) {
    headers.remove("host");
}

fn blocked_method_response(id: &Value, method: &str) -> Response {
    let payload = json!({
        "jsonrpc": "2.0",
        "error": {
            "code": -32601,
            "message": format!("Method '{method}' blocked by veto proxy"),
        },
        "id": id.clone(),
    });

    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .expect("valid blocked response")
}

fn build_target_uri(base: &Uri, incoming: &Uri) -> Result<Uri, ProxyError> {
    let mut parts = base.clone().into_parts();
    if let Some(path_and_query) = incoming.path_and_query() {
        parts.path_and_query = Some(path_and_query.clone());
    }
    Uri::from_parts(parts).map_err(ProxyError::BadUpstreamUri)
}

#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    #[serde(default)]
    method: String,
    #[serde(default)]
    id: Value,
}

fn parse_json_rpc(body: &[u8]) -> Result<JsonRpcRequest, HandlerErrorKind> {
    if body.is_empty() {
        return Err(HandlerErrorKind::InvalidRequest("empty body".into()));
    }

    let value: Value = serde_json::from_slice(body).map_err(HandlerErrorKind::InvalidJson)?;

    let request = match value {
        Value::Object(_) => serde_json::from_value(value).map_err(HandlerErrorKind::InvalidJson)?,
        Value::Array(_) => {
            return Err(HandlerErrorKind::Unsupported(
                "batch requests not supported".into(),
            ));
        }
        _ => {
            return Err(HandlerErrorKind::InvalidRequest(
                "JSON-RPC payload must be an object".into(),
            ));
        }
    };

    if request.method.trim().is_empty() {
        return Err(HandlerErrorKind::InvalidRequest(
            "JSON-RPC method is required".into(),
        ));
    }

    Ok(request)
}

#[derive(Debug)]
enum HandlerError {
    JsonRpc(Response),
    Internal(ProxyError),
}

impl From<ProxyError> for HandlerError {
    fn from(error: ProxyError) -> Self {
        HandlerError::Internal(error)
    }
}

#[derive(Debug)]
enum HandlerErrorKind {
    InvalidJson(serde_json::Error),
    InvalidRequest(String),
    Unsupported(String),
}

impl From<HandlerErrorKind> for HandlerError {
    fn from(kind: HandlerErrorKind) -> Self {
        HandlerError::JsonRpc(json_rpc_error_response(kind))
    }
}

fn json_rpc_error_response(kind: HandlerErrorKind) -> Response {
    let message = match &kind {
        HandlerErrorKind::InvalidJson(error) => error.to_string(),
        HandlerErrorKind::InvalidRequest(msg) | HandlerErrorKind::Unsupported(msg) => msg.clone(),
    };

    let payload = json!({
        "jsonrpc": "2.0",
        "error": {
            "code": -32600,
            "message": message,
        },
        "id": Value::Null,
    });

    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("content-type", "application/json")
        .body(Body::from(payload.to_string()))
        .expect("valid json error response")
}

/// Errors surfaced by the proxy runtime.
#[derive(Debug, Error)]
pub enum ProxyError {
    /// Failed to bind to the requested socket.
    #[error("failed to bind proxy socket: {0}")]
    Bind(std::io::Error),
    /// Axum server error.
    #[error("server error: {0}")]
    Server(std::io::Error),
    /// Body extraction failure.
    #[error("failed to read request body: {0}")]
    Body(Box<dyn std::error::Error + Send + Sync>),
    /// Failed to reach upstream.
    #[error("upstream request failed: {0}")]
    Upstream(hyper::Error),
    /// Failed to construct upstream URI for forwarding.
    #[error("failed to construct upstream URI: {0}")]
    BadUpstreamUri(http::Error),
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        error!("failed to listen for shutdown signal: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::routing::post;
    use axum::{Json, Router, body::Body};
    use http::{Request, StatusCode};
    use hyper_util::client::legacy::Client;
    use hyper_util::client::legacy::connect::HttpConnector;
    use hyper_util::rt::TokioExecutor;
    use rstest::rstest;
    use serde_json::json;
    use std::collections::HashSet;
    use std::net::SocketAddr;

    #[tokio::test]
    async fn blocked_method_response_contains_message() {
        let response = blocked_method_response(&Value::from(1), "eth_sendTransaction");
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = http_body_util::BodyExt::collect(response.into_body())
            .await
            .unwrap()
            .to_bytes();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["error"]["code"], -32601);
        assert!(
            value["error"]["message"]
                .as_str()
                .unwrap()
                .contains("blocked")
        );
        assert_eq!(value["id"], 1);
    }

    #[rstest]
    fn build_uri_uses_incoming_path() {
        let base = "http://127.0.0.1:8545".parse::<Uri>().unwrap();
        let incoming = "http://localhost:3000/custom".parse::<Uri>().unwrap();
        let result = build_target_uri(&base, &incoming).unwrap();
        assert_eq!(result.to_string(), "http://127.0.0.1:8545/custom");
    }

    #[tokio::test]
    async fn end_to_end_blocks_configured_method() {
        let (upstream, upstream_handle) = spawn_echo_server().await;
        let config = Config::new(
            "127.0.0.1:0".parse().unwrap(),
            format!("http://{}", upstream).parse().unwrap(),
            HashSet::from([String::from("eth_sendtransaction")]),
        );

        let listener = TcpListener::bind(config.bind_address())
            .await
            .expect("bind");
        let bind_addr = listener.local_addr().unwrap();

        let state = AppState::try_from_config(config).unwrap();
        let router = Router::new().fallback(any(proxy_handler)).with_state(state);

        let server = tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(async {})
                .await
                .unwrap();
        });

        let mut connector = HttpConnector::new();
        connector.enforce_http(false);
        let client = Client::builder(TokioExecutor::new()).build(connector);
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "eth_sendTransaction",
            "params": []
        });

        let uri = format!("http://{bind_addr}/").parse().unwrap();
        let request = Request::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(payload.to_string()))
            .unwrap();
        let response = client.request(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = http_body_util::BodyExt::collect(response.into_body())
            .await
            .unwrap()
            .to_bytes();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["error"]["code"], -32601);

        server.abort();
        upstream_handle.abort();
    }

    async fn spawn_echo_server() -> (SocketAddr, tokio::task::JoinHandle<()>) {
        let router = Router::new().route(
            "/",
            post(|Json(value): Json<Value>| async move { Json(value) }),
        );

        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(async {})
                .await
                .unwrap();
        });
        (addr, handle)
    }
}
