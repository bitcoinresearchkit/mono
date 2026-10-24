//! Bounded submission admission before buffering the request body.

use crate::request_state::RequestState;
use aide::OperationInput;
use axum::{
    extract::FromRequestParts,
    http::request::Parts,
    response::{IntoResponse, Response},
};
use bitcoin::Weight as BitcoinWeight;
use tokio::sync::OwnedSemaphorePermit;

use crate::{AppState, Error, error::Result, extended::HeaderMapExtended, params::Empty};

// Serialized transaction bytes cannot exceed the block weight, and hex doubles them. The bound
// includes surrounding whitespace, which is trimmed before validation.
pub const MAX_BODY_BYTES: usize = BitcoinWeight::MAX_BLOCK.to_wu() as usize * 2;

pub struct BroadcastPermit(OwnedSemaphorePermit);

impl BroadcastPermit {
    // One active RPC and up to three admitted requests waiting asynchronously.
    pub const CAPACITY: usize = 4;
}

impl FromRequestParts<AppState> for BroadcastPermit {
    type Rejection = Error;

    async fn from_request_parts(_: &mut Parts, state: &AppState) -> Result<Self> {
        state
            .broadcast_requests
            .clone()
            .try_acquire_owned()
            .map(Self)
            .map_err(|_| Error::overloaded("too many pending transaction submissions"))
    }
}

impl OperationInput for BroadcastPermit {}

fn validate_hex(body: &str) -> Result<&str> {
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

pub async fn serve(
    permit: BroadcastPermit,
    _: Empty,
    RequestState(state): RequestState,
    body: String,
) -> Result<Response> {
    let hex = validate_hex(&body)?;
    let result = state.node.send_raw_transaction(hex).await;
    drop(permit.0);
    // A submission's answer is never reused, like its errors.
    let mut response = result.map_err(Error::from)?.to_string().into_response();
    let headers = response.headers_mut();
    headers.insert_cache_control("no-store");
    headers.insert_cdn_cache_control("no-store");
    Ok(response)
}
