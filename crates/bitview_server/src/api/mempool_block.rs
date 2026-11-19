use axum::{body::Bytes, http::HeaderMap, response::Response};
use brk_types::{MempoolBlockHash, MempoolBlockIndex};

use crate::{
    AppState,
    answer::{Answer, Content, JSON},
};

/// A projected mempool block with its transactions in their light shape (under 1 MB).
pub async fn serve(state: AppState, headers: &HeaderMap, index: MempoolBlockIndex) -> Response {
    state
        .respond(headers, state.mempool_block(), move |q| {
            let source = q.resolve_mempool_block(index)?;
            let tag = format!("mempool-block-{index}-{}", source.hash());
            Ok(Answer::new(tag, move |_| {
                Ok(Content::new(JSON, Bytes::from_owner(source.json())))
            }))
        })
        .await
}

/// A projected mempool block's changes since the client's copy, `since`.
pub async fn serve_diff(
    state: AppState,
    headers: &HeaderMap,
    index: MempoolBlockIndex,
    since: MempoolBlockHash,
) -> Response {
    state
        .respond(headers, state.mempool_block_diff(), move |q| {
            let resolved = q.resolve_mempool_block_diff(index, since)?;
            let tag = format!(
                "mempool-block-{index}-diff-{}-{}",
                resolved.since(),
                resolved.source().hash()
            );
            Ok(Answer::new(tag, move |_| match resolved.full_json() {
                Some(json) => Ok(Content::new(JSON, Bytes::from_owner(json))),
                None => Content::json(&resolved.build()),
            }))
        })
        .await
}
