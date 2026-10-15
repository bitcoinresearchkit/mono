use axum::{http::HeaderMap, response::Response};

use crate::{AppState, CacheParams, error::Result, extended::ResponseExtended};

pub async fn serve(state: AppState, headers: HeaderMap) -> Result<Response> {
    // This hash read validates completed publication before a cheap 304.
    let hash = state.query.read(|q| Ok(q.mempool_txids()?.hash())).await?;
    let params = CacheParams::live_hash(hash);
    if params.matches_etag(&headers) {
        return Ok(Response::new_not_modified(&params));
    }
    Ok(state
        .respond_admitted(
            &state.sync_query,
            &state.mempool_txid_bodies,
            headers,
            "application/json",
            |query| {
                // Recheck publication after admission; the array comes from the same one.
                let txids = query.mempool_txids()?;
                let params = CacheParams::live_hash(txids.hash());
                Ok((txids, params))
            },
            |_, txids| Ok(txids.json().into()),
        )
        .await?)
}
