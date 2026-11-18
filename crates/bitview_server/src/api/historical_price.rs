use axum::{body::Bytes, http::HeaderMap, response::Response};
use bitview_query::{RepresentationId, Result as QueryResult};
use brk_types::{Timestamp, Version};
use serde_json::to_vec;

use crate::{
    AppState,
    answer::{Answer, Content, JSON},
    etag::Etag,
};

pub async fn serve(state: AppState, headers: &HeaderMap, timestamp: Option<Timestamp>) -> Response {
    let cache = state.historical_price_cache.clone();
    state
        .respond(headers, state.historical_price(), move |query| {
            let source = query.resolve_historical_price()?;
            let content = || -> QueryResult<_> {
                let (price, settled) = source.get(timestamp)?;
                let bytes = Bytes::from(to_vec(&price)?);
                let identity = RepresentationId::content(&bytes);
                Ok((bytes, identity, settled))
            };
            // Point responses bypass the cache rather than evicting the full history.
            let (bytes, identity, settled) = if timestamp.is_none() {
                let (bytes, identity) = cache.get_or_try_init(source.revision(), || {
                    content().map(|(bytes, identity, _)| (bytes, identity))
                })?;
                (bytes, identity, false)
            } else {
                content()?
            };
            let tag = Etag::representation(Version::ONE, identity);
            Ok(Answer::ready(tag, Content::new(JSON, bytes)).settled(settled))
        })
        .await
}
