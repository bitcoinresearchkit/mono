use axum::{body::Bytes, http::HeaderMap, response::Response};
use brk_types::{Txid, Version};

use crate::{AppState, CacheParams, error::Result, extended::ResponseExtended, params::page_size};

pub async fn serve_page(
    state: AppState,
    headers: HeaderMap,
    after: Option<Txid>,
    max_txs: Option<usize>,
) -> Response {
    state.respond_json_bound(headers, Version::ONE, move |q| {
        q.mempool_txids_page_json(after.as_ref(), page_size(max_txs, 10_000))
    }).await
}

pub async fn serve(state: AppState, headers: HeaderMap) -> Result<Response> {
    // This hash read validates completed publication before a cheap 304.
    let hash = state.query.read(|q| Ok(q.mempool_txids()?.hash())).await?;
    let params = CacheParams::live_hash(hash);
    if params.etag.matches(&headers) {
        return Ok(Response::new_not_modified(&params));
    }
    Ok(state
        .respond_admitted(
            &state.chain_query,
            &state.mempool_txid_bodies,
            headers,
            "application/json",
            |query| {
                // Recheck publication after admission; the array comes from the same one.
                let txids = query.mempool_txids()?;
                let params = CacheParams::live_hash(txids.hash());
                Ok((txids, params))
            },
            |_, txids| Ok(Bytes::from_owner(txids.json())),
        )
        .await?)
}
