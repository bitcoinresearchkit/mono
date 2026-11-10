//! Bounded submission admission before buffering the request body.

use aide::OperationInput;
use axum::{
    extract::{FromRef, FromRequestParts, Query},
    http::{Method, StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use bitcoin::Weight as BitcoinWeight;
use brk_types::Txid;
use brk_rpc::AsyncClient;
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::{AppState, Error, error::Result, extended::HeaderMapExtended, params::Empty};
use crate::request_action::RequestAction;

// Serialized transaction bytes cannot exceed the block weight, and hex doubles them. The bound
// includes surrounding whitespace, which is trimmed before validation.
pub const MAX_BODY_BYTES: usize = BitcoinWeight::MAX_BLOCK.to_wu() as usize * 2;

#[derive(Clone)]
pub(super) struct BroadcastState {
    node: AsyncClient,
    requests: Arc<Semaphore>,
}

impl FromRef<AppState> for BroadcastState {
    fn from_ref(state: &AppState) -> Self {
        Self { node: state.node.clone(), requests: state.broadcast_requests.clone() }
    }
}

pub struct BroadcastPermit {
    permit: OwnedSemaphorePermit,
    node: AsyncClient,
}

impl BroadcastPermit {
    // One active RPC and up to three admitted requests waiting asynchronously.
    pub const CAPACITY: usize = 4;
}

impl<S: Send + Sync> FromRequestParts<S> for BroadcastPermit
where BroadcastState: FromRef<S>,
{
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self> {
        // Axum serves HEAD through GET by default; it must never submit a transaction.
        if parts.method != Method::GET && parts.method != Method::POST {
            return Err(Error::new(StatusCode::METHOD_NOT_ALLOWED,
                bitview_types::ErrorCode::MethodNotAllowed, "transaction submission requires GET or POST"));
        }
        if let Some(action) = parts.extensions.get::<RequestAction>() {
            action.mark();
        }
        let state = BroadcastState::from_ref(state);
        state.requests
            .try_acquire_owned()
            .map(|permit| Self { permit, node: state.node })
            .map_err(|_| Error::overloaded("too many pending transaction submissions"))
    }
}

impl OperationInput for BroadcastPermit {}

fn validate_hex(body: &str) -> Result<&str> {
    if body.len() > MAX_BODY_BYTES {
        return Err(Error::bad_request("transaction exceeds 8,000,000 hexadecimal bytes"));
    }
    let hex = body.trim();
    if hex.is_empty()
        || !hex.len().is_multiple_of(2)
        || !hex.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(Error::bad_request(
            "transaction body must contain nonempty, even-length hexadecimal",
        ));
    }
    Ok(hex)
}

/// Submits a raw transaction, as hex, to the node: its txid once the node accepts it.
pub async fn submit(permit: BroadcastPermit, body: &str) -> Result<Txid> {
    let hex = validate_hex(body)?;
    let result = permit.node.send_raw_transaction(hex).await;
    drop(permit.permit);
    result.map_err(Error::from)
}

pub async fn serve(
    permit: BroadcastPermit,
    _: Empty,
    body: String,
) -> Result<Response> {
    respond(permit, &body).await
}

#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct BroadcastQuery {
    /// Raw transaction as hexadecimal text. Request-line limits also apply.
    tx: String,
}

pub(super) async fn serve_query(
    permit: BroadcastPermit,
    Query(query): Query<BroadcastQuery>,
) -> Result<Response> {
    respond(permit, &query.tx).await
}

async fn respond(permit: BroadcastPermit, body: &str) -> Result<Response> {
    // A submission's answer is never reused, like its errors.
    let mut response = submit(permit, body).await?.to_string().into_response();
    let headers = response.headers_mut();
    headers.insert_cache_control("no-store");
    headers.insert_cdn_cache_control("no-store");
    Ok(response)
}

#[cfg(test)]
mod tests;
