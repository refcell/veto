use axum::body::Body;
use axum::http::StatusCode;
use axum::response::Response;
use serde::Deserialize;
use serde_json::{Value, json};

/// Inbound JSON-RPC request payload.
#[derive(Debug, Deserialize)]
pub(crate) struct JsonRpcRequest {
    #[serde(default)]
    pub method: String,
    #[serde(default)]
    pub id: Value,
}

/// Errors produced while decoding JSON-RPC payloads.
#[derive(Debug)]
pub(crate) enum JsonRpcError {
    InvalidJson(serde_json::Error),
    InvalidRequest(String),
    Unsupported(String),
}

/// Parse bytes into a [`JsonRpcRequest`], validating the shape along the way.
pub(crate) fn parse_json_rpc(body: &[u8]) -> Result<JsonRpcRequest, JsonRpcError> {
    if body.is_empty() {
        return Err(JsonRpcError::InvalidRequest("empty body".into()));
    }

    let value: Value = serde_json::from_slice(body).map_err(JsonRpcError::InvalidJson)?;

    let request: JsonRpcRequest = match value {
        Value::Object(_) => serde_json::from_value(value).map_err(JsonRpcError::InvalidJson)?,
        Value::Array(_) => {
            return Err(JsonRpcError::Unsupported(
                "batch requests not supported".into(),
            ));
        }
        _ => {
            return Err(JsonRpcError::InvalidRequest(
                "JSON-RPC payload must be an object".into(),
            ));
        }
    };

    if request.method.trim().is_empty() {
        return Err(JsonRpcError::InvalidRequest(
            "JSON-RPC method is required".into(),
        ));
    }

    Ok(request)
}

/// Convert a [`JsonRpcError`] into a JSON-RPC response payload.
pub(crate) fn error_response(error: JsonRpcError) -> Response {
    let message = match &error {
        JsonRpcError::InvalidJson(err) => err.to_string(),
        JsonRpcError::InvalidRequest(message) | JsonRpcError::Unsupported(message) => {
            message.clone()
        }
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
