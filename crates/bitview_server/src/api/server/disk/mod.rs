use axum::{http::HeaderMap, response::Response};
use bitview_types::DiskUsage;

use crate::{answer::Answer, params::Empty, request_state::RequestState};

mod walk;

pub async fn get(headers: HeaderMap, _: Empty, RequestState(state): RequestState) -> Response {
    let shared = state.clone();
    state
        .respond(&headers, state.chain(), move |q| {
            // Nearly all the files are the indexer stores' tables, whose bytes the stores track
            // in memory, and the node reports its own block files' bytes: only what is left of
            // Bitview's directory is walked, a few hundred entries. The server's own logs grow
            // with every request and are not counted.
            let (stores_path, stores_bytes) = q.stores_disk();
            let walked = walk::dir_size(&shared.data_path, &[&shared.logs_path, stores_path])?;
            let brk_bytes = walk::sum_bytes(walked, stores_bytes)?;
            let bitcoin_bytes = q.node_blocks_bytes();
            // These totals determine every DiskUsage field. Bump the representation
            // version if its fields, derivations or formatting change.
            let tag = format!("disk2-{brk_bytes}-{bitcoin_bytes}");
            Ok(Answer::json(tag, move |_| {
                Ok(DiskUsage::new(brk_bytes, bitcoin_bytes))
            }))
        })
        .await
}
