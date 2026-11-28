//! Transaction submission, admitted once the request body has arrived.

use crate::request_state::RequestState;
use axum::response::{IntoResponse, Response};
use bitcoin::policy::MAX_STANDARD_TX_WEIGHT;
use bitview_http::CachePolicy;
use brk_types::Txid;
use tokio::time::{Instant as TimeInstant, timeout_at};

use crate::{AppState, Error, error::Result, params::Empty};

// The node takes no transaction over the standard weight, a serialized byte weighs at least one
// unit, and hex doubles them: a larger body could only be refused after buffering it. The bound
// includes surrounding whitespace, which is trimmed before validation.
pub const MAX_BODY_BYTES: usize = MAX_STANDARD_TX_WEIGHT as usize * 2;

// Submissions with the node at once, each on its own node connection.
pub const CAPACITY: usize = 4;

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

/// Submits a raw transaction, as hex, to the node: its txid once the node accepts it. Admission
/// is taken here, with the body already read: a client sending it slowly holds no place.
pub async fn submit(state: &AppState, body: &str) -> Result<Txid> {
    let hex = validate_hex(body)?;
    let permit = state.broadcast_requests.try_acquire().map_err(|_| {
        Error::overloaded("transaction not submitted: too many pending submissions")
    })?;
    // Only while the node holds the submission is its outcome unknown, so only this wait says so:
    // the request's deadline is applied here too, and answers before the request's own timeout
    // (`request_deadline`, which polls this handler first) with its plain one.
    let deadline = TimeInstant::from_std(state.query.read_deadline());
    let result = timeout_at(deadline, state.node.send_raw_transaction(hex)).await;
    drop(permit);
    result
        .map_err(|_| Error::submission_timeout())?
        .map_err(Error::from)
}

pub async fn serve(
    _: Empty,
    RequestState(state): RequestState,
    body: String,
) -> Result<Response> {
    // A submission's answer is never reused, like its errors.
    let mut response = submit(&state, &body).await?.to_string().into_response();
    CachePolicy::NoStore.apply(response.headers_mut());
    Ok(response)
}
