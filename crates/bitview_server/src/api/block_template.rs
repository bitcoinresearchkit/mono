use std::sync::Arc;

use axum::{http::HeaderMap, response::Response};
use bitview_query::{Query, Result as QueryResult};
use brk_types::NextBlockHash;
use serde::Serialize;
use serde_json::to_vec;
use tokio::sync::Semaphore;

use crate::{AppState, CacheParams, error::Result, extended::ResponseExtended};

/// The projected next block (about 10 MB).
pub async fn serve(state: AppState, headers: HeaderMap) -> Result<Response> {
    serve_json(
        &state,
        headers,
        &state.block_template_bodies,
        |query| {
            let source = query.resolve_block_template()?;
            let params = CacheParams::live(format!("template-v2-{}", source.hash()?));
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
    serve_json(
        &state,
        headers,
        &state.block_template_diff_bodies,
        move |query| {
            let resolved = query.resolve_block_template_diff(since)?;
            let params = CacheParams::live(format!(
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

/// Encodes what `select` resolves under the `bodies` budget, retained until the body is sent: slow
/// readers would otherwise each hold one while new requests build more. A matching conditional
/// request answers 304 before waiting for admission.
async fn serve_json<S: 'static, T: Serialize>(
    state: &AppState,
    headers: HeaderMap,
    bodies: &Arc<Semaphore>,
    select: impl Fn(&Query) -> QueryResult<(S, CacheParams)> + Clone + Send + 'static,
    build: impl Fn(S) -> brk_error::Result<T> + Clone + Send + 'static,
) -> Result<Response> {
    let check = select.clone();
    let params = state.query.read(move |query| Ok(check(query)?.1)).await?;
    if params.matches_etag(&headers) {
        return Ok(Response::new_not_modified(&params));
    }
    // Resolved again after admission: the template may have moved on.
    Ok(state
        .respond_admitted(
            &state.sync_query,
            bodies,
            headers,
            "application/json",
            select,
            move |_, selected| Ok(to_vec(&build(selected)?)?.into()),
        )
        .await?)
}
