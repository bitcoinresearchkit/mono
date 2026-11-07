use std::sync::Arc;

use axum::{body::Bytes, http::HeaderMap, response::Response};
use bitview_query::{Query, Result as QueryResult};
use brk_types::{MempoolBlockHash, MempoolBlockIndex};
use serde_json::to_vec;
use tokio::sync::Semaphore;

use crate::{AppState, CacheParams, error::Result, extended::ResponseExtended};

/// A projected mempool block with its transactions (about 10 MB each).
pub async fn serve(
    state: AppState,
    headers: HeaderMap,
    index: MempoolBlockIndex,
) -> Result<Response> {
    serve_json(
        &state,
        headers,
        &state.mempool_block_bodies,
        move |query| {
            let source = query.resolve_mempool_block(index)?;
            let params = CacheParams::live(format!("mempool-block-{index}-{}", source.hash()));
            Ok((source, params))
        },
        |source| Ok(Bytes::from_owner(source.json())),
    )
    .await
}

/// A projected mempool block's changes since the client's copy, `since`.
pub async fn serve_diff(
    state: AppState,
    headers: HeaderMap,
    index: MempoolBlockIndex,
    since: MempoolBlockHash,
) -> Result<Response> {
    serve_json(
        &state,
        headers,
        &state.mempool_block_diff_bodies,
        move |query| {
            let resolved = query.resolve_mempool_block_diff(index, since)?;
            let params = CacheParams::live(format!(
                "mempool-block-{index}-diff-{}-{}",
                resolved.since(),
                resolved.source().hash()
            ));
            Ok((resolved, params))
        },
        |resolved| Ok(to_vec(&resolved.build())?.into()),
    )
    .await
}

/// Encodes what `select` resolves under the `bodies` budget, retained until the body is sent: slow
/// readers would otherwise each hold one while new requests build more. A matching conditional
/// request answers 304 before waiting for admission.
async fn serve_json<S: 'static>(
    state: &AppState,
    headers: HeaderMap,
    bodies: &Arc<Semaphore>,
    select: impl Fn(&Query) -> QueryResult<(S, CacheParams)> + Clone + Send + 'static,
    build: impl Fn(S) -> brk_error::Result<Bytes> + Clone + Send + 'static,
) -> Result<Response> {
    let check = select.clone();
    let params = state.query.read(move |query| Ok(check(query)?.1)).await?;
    if params.etag.matches(&headers) {
        return Ok(Response::new_not_modified(&params));
    }
    // Resolved again after admission: the template may have moved on.
    Ok(state
        .respond_admitted(
            &state.chain_query,
            bodies,
            headers,
            "application/json",
            select,
            move |_, selected| Ok(build(selected)?),
        )
        .await?)
}
