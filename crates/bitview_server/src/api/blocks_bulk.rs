use aide::axum::{ApiRouter, routing::get_with};
use axum::{extract::Path, http::HeaderMap, response::Response};
use bitview_types::BlockBulk;
use brk_types::Height;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    AppState, CacheParams, Error,
    api::{blocks::V1_BLOCK_SCHEMA, tags},
    extended::{ResponseExtended, TransformResponseExtended},
    params::Empty,
    request_state::RequestState,
};

#[derive(Deserialize, JsonSchema)]
struct BulkFrom {
    #[serde(deserialize_with = "bitview_types::de_plain")]
    from: Height,
}

#[derive(Deserialize, JsonSchema)]
struct BulkRange {
    #[serde(deserialize_with = "bitview_types::de_plain")]
    from: Height,
    #[serde(deserialize_with = "bitview_types::de_plain")]
    to: Height,
}

impl AppState {
    async fn respond_blocks_bulk(
        &self,
        headers: HeaderMap,
        from: Height,
        to: Option<Height>,
    ) -> Result<Response, Error> {
        let cdn_cache_mode = self.cdn_cache_mode;
        Ok(self.read_admitted(move |q| {
            let snapshot = q.resolve_blocks_bulk(from, to)?;
            // No price input: a price-only publication cannot change this body.
            let params = CacheParams::live(format!(
                "blocks-bulk-v1-{V1_BLOCK_SCHEMA}-{from}-{}", snapshot.identity(),
            )).stable_if(to.is_some() && snapshot.is_settled(), cdn_cache_mode);
            if params.etag.matches(&headers) {
                return Ok(ResponseExtended::new_not_modified(&params));
            }
            let rows = snapshot.build(q)?;
            Ok(Response::cached_json(&headers, &params, || rows))
        }).await?)
    }
}

pub(super) fn add_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router.api_route(
        "/api/v1/blocks-bulk/{from}",
        get_with(
            async |headers: HeaderMap, Path(path): Path<BulkFrom>, _: Empty,
                   RequestState(state): RequestState| {
                state.respond_blocks_bulk(headers, path.from, None).await
            },
            |op| {
                op.id("get_blocks_bulk_to_tip")
                    .tag(tags::BLOCKS)
                    .summary("Bulk blocks to tip")
                    .description("Flat snake_case rows in ascending height order from `from` through the published tip, inclusive. At most 10 blocks; no query parameters. Uses local V1 analytics: effective rate distributions are vsize-weighted and can differ from mempool.space.")
                    .json_response::<Vec<BlockBulk>>()
                    .not_modified()
                    .not_found()
            },
        ),
    ).api_route(
        "/api/v1/blocks-bulk/{from}/{to}",
        get_with(
            async |headers: HeaderMap, Path(path): Path<BulkRange>, _: Empty,
                   RequestState(state): RequestState| {
                state.respond_blocks_bulk(headers, path.from, Some(path.to)).await
            },
            |op| {
                op.id("get_blocks_bulk")
                    .tag(tags::BLOCKS)
                    .summary("Bulk blocks")
                    .description("Flat snake_case rows in ascending height order from `from` through `to`, inclusive. At most 10 blocks; a missing endpoint fails the request. Uses local V1 analytics: effective rate distributions are vsize-weighted and can differ from mempool.space.")
                    .json_response::<Vec<BlockBulk>>()
                    .not_modified()
                    .not_found()
            },
        ),
    )
}
