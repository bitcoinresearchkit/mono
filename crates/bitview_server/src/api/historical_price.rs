use axum::{body::Bytes, http::HeaderMap, response::Response};
use bitview_query::{RepresentationId, Result as QueryResult};
use brk_types::{Timestamp, Version};
use serde_json::to_vec;

use crate::{AppState, CacheParams, error::Result};

pub async fn serve(
    state: AppState,
    headers: HeaderMap,
    timestamp: Option<Timestamp>,
) -> Result<Response> {
    let cache = state.historical_price_cache.clone();
    Ok(state
        .respond_admitted(
            &state.sync_query,
            &state.historical_price_bodies,
            headers,
            "application/json",
            move |query| {
                let source = query.resolve_historical_price()?;
                let content = || -> QueryResult<_> {
                    let bytes = Bytes::from(to_vec(&source.get(timestamp)?)?);
                    let identity = RepresentationId::content(&bytes);
                    Ok((bytes, identity))
                };
                // Point responses bypass the cache rather than evicting the full history.
                let (bytes, identity) = if timestamp.is_none() {
                    cache.get_or_try_init(source.revision(), content)?
                } else {
                    content()?
                };
                Ok((bytes, CacheParams::representation(Version::ONE, identity)))
            },
            |_, bytes| Ok(bytes),
        )
        .await?)
}
