use std::sync::Arc;

use axum::{http::HeaderMap, response::Response};
use bitview_query::{Query, Result as QueryResult};
use brk_types::NextBlockHash;
use serde::Serialize;
use serde_json::to_vec;
use tokio::sync::Semaphore;

use crate::{
    AppState, CacheParams, CacheStrategy, CdnCacheMode,
    error::Result,
    extended::{HeaderMapExtended, ResponseExtended},
    raw_body::RawBodyPermit,
};

/// The projected next block (about 10 MB).
pub async fn serve(state: AppState, headers: HeaderMap) -> Result<Response> {
    let bodies = state.block_template_bodies.clone();
    serve_json(
        state,
        headers,
        bodies,
        |query| {
            let source = query.resolve_block_template()?;
            let params = live(format!("template-v2-{}", source.hash()?));
            Ok((source, params))
        },
        |source| source.build(),
    )
    .await
}

/// The projected next block's changes since `since`.
pub async fn serve_diff(
    state: AppState,
    headers: HeaderMap,
    since: NextBlockHash,
) -> Result<Response> {
    let bodies = state.block_template_diff_bodies.clone();
    serve_json(
        state,
        headers,
        bodies,
        move |query| {
            let resolved = query.resolve_block_template_diff(since)?;
            let params = live(format!(
                "template-diff-v2-{}-{}",
                resolved.since(),
                resolved.source().hash()?
            ));
            Ok((resolved, params))
        },
        |resolved| resolved.build(),
    )
    .await
}

fn live(key: String) -> CacheParams {
    CacheParams::resolve(&CacheStrategy::Live(key.into()), CdnCacheMode::Live)
}

/// Encodes what `select` resolves under the `bodies` budget, retained until the body is sent: slow
/// readers would otherwise each hold one while new requests build more. A matching conditional
/// request answers 304 before waiting for admission.
async fn serve_json<S, T: Serialize>(
    state: AppState,
    headers: HeaderMap,
    bodies: Arc<Semaphore>,
    select: impl Fn(&Query) -> QueryResult<(S, CacheParams)> + Clone + Send + 'static,
    build: impl Fn(S) -> brk_error::Result<T> + Clone + Send + 'static,
) -> Result<Response> {
    let check = select.clone();
    let params = state.query.read(move |query| Ok(check(query)?.1)).await?;
    if params.matches_etag(&headers) {
        return Ok(Response::new_not_modified(&params));
    }
    state
        .read_body(&state.sync_query, &bodies.clone(), move |query, permit| {
            // Resolved again after admission: the template may have moved on.
            let (selected, params) = select(query)?;
            if params.matches_etag(&headers) {
                return Ok(Some(Response::new_not_modified(&params)));
            }
            let Some(permit) = permit.or_else(|| RawBodyPermit::try_acquire(&bodies)) else {
                return Ok(None);
            };
            let bytes = to_vec(&build(selected)?)?;
            Ok(Some(permit.response(
                params,
                bytes.into(),
                HeaderMapExtended::insert_content_type_application_json,
            )))
        })
        .await
        .map_err(Into::into)
}
