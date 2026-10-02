//! Bounded submission admission before buffering the request body.

use crate::request_state::RequestState;
use aide::OperationInput;
use axum::{extract::FromRequestParts, http::request::Parts};
use bitcoin::Weight as BitcoinWeight;
use tokio::sync::OwnedSemaphorePermit;

use crate::{AppState, Error, error::Result, params::Empty};

// A transaction cannot exceed the block weight in serialized bytes. The bound
// includes surrounding whitespace; trim is retained for ordinary pasted hex.
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
) -> Result<String> {
    let hex = validate_hex(&body)?;
    let result = state
        .node
        .send_raw_transaction(hex)
        .await
        .map(|txid| txid.to_string());
    drop(permit.0);
    result.map_err(Error::from)
}
