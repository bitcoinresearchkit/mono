use axum::{http::HeaderMap, response::Response};

use crate::{
    AppState, CacheParams, CacheStrategy, CdnCacheMode,
    error::Result,
    extended::{HeaderMapExtended, ResponseExtended},
    raw_body::RawBodyPermit,
};

pub async fn serve(state: AppState, headers: HeaderMap) -> Result<Response> {
    // This hash read validates completed publication before a cheap 304.
    let hash = state.query.read(|q| Ok(q.mempool_txids()?.hash())).await?;
    let params = CacheParams::resolve(&CacheStrategy::LiveHash(hash), CdnCacheMode::Live);
    if params.matches_etag(&headers) {
        return Ok(Response::new_not_modified(&params));
    }
    let bodies = state.mempool_txid_bodies.clone();
    state
        .read_body(
            &state.sync_query,
            &state.mempool_txid_bodies,
            move |query, permit| {
                // Recheck publication after admission; the array comes from the same one.
                let txids = query.mempool_txids()?;
                let params = CacheParams::resolve(
                    &CacheStrategy::LiveHash(txids.hash()),
                    CdnCacheMode::Live,
                );
                if params.matches_etag(&headers) {
                    return Ok(Some(Response::new_not_modified(&params)));
                }
                let Some(permit) = permit.or_else(|| RawBodyPermit::try_acquire(&bodies)) else {
                    return Ok(None);
                };
                Ok(Some(permit.response(
                    params,
                    txids.json().into(),
                    HeaderMapExtended::insert_content_type_application_json,
                )))
            },
        )
        .await
        .map_err(Into::into)
}
