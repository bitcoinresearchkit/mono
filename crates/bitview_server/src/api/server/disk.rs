use axum::{http::HeaderMap, response::Response};
use bitview_types::DiskUsage;

use crate::{answer::Answer, params::Empty, request_state::RequestState};

pub async fn get(headers: HeaderMap, _: Empty, RequestState(state): RequestState) -> Response {
    state
        .respond(&headers, state.chain(), |q| {
            // Both totals are the daemon's records, taken at start, at each publication and at
            // each new tip: no request reads the filesystem or asks the node.
            let (brk_bytes, bitcoin_bytes) = (q.data_bytes(), q.node_blocks_bytes());
            // These totals determine every DiskUsage field. Bump the representation
            // version if its fields, derivations or formatting change.
            let tag = format!("disk2-{brk_bytes}-{bitcoin_bytes}");
            Ok(Answer::json(tag, move |_| {
                Ok(DiskUsage::new(brk_bytes, bitcoin_bytes))
            }))
        })
        .await
}
