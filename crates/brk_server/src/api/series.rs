//! Live `/api/series/*` API: catalog, search, info, single-series, bulk.
//!
//! Holds the shared `serve` helper used by every series endpoint that returns
//! a formatted body (single + raw + bulk + the legacy module's deprecated
//! handler in `series_legacy.rs`).

use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, Uri},
    response::{IntoResponse, Response},
};
use brk_error::Result as BrkResult;
use brk_query::{Query as BrkQuery, ResolvedQuery};
use brk_traversable::TreeNode;
use brk_types::{
    DataRangeFormat, DetailedSeriesCount, Format, IndexInfo, Output, PaginatedSeries, Pagination,
    SearchQuery, SeriesData, SeriesInfo, SeriesNameWithIndex, SeriesOutput, SeriesSelection,
    Version,
};

use crate::{
    AppState, CacheParams, CacheStrategy, Result,
    extended::{HeaderMapExtended, TransformResponseExtended},
    params::{Empty, SeriesParam},
};

/// Shared response pipeline for every series endpoint.
///
/// Resolves the query (which determines the cache key), then delegates to
/// [`AppState::respond_with_params`] for the etag short-circuit, server-side
/// cache lookup, body formatting, and header assembly.
pub(super) async fn serve(
    state: AppState,
    uri: Uri,
    headers: HeaderMap,
    params: SeriesSelection,
    to_bytes: impl FnOnce(&BrkQuery, ResolvedQuery) -> BrkResult<Bytes> + Send + 'static,
) -> Result<Response> {
    let max_weight = state.max_weight;
    let resolved = state.run(move |q| q.resolve(params, max_weight)).await?;

    let format = resolved.format;
    let csv_filename = resolved.csv_filename();
    let cache_params = CacheParams::series(
        resolved.version,
        resolved.start,
        resolved.end,
        resolved.stable_count,
        resolved.hash_prefix,
    );

    Ok(state
        .respond_with_params(
            &headers,
            &uri,
            cache_params,
            move |h| match format {
                Format::CSV => {
                    h.insert_content_disposition_attachment(&csv_filename);
                    h.insert_content_type_text_csv();
                }
                Format::JSON => h.insert_content_type_application_json(),
            },
            move |q| to_bytes(q, resolved),
        )
        .await)
}

fn output_to_bytes(out: SeriesOutput) -> BrkResult<Bytes> {
    Ok(match out.output {
        Output::CSV(s) => Bytes::from(s),
        Output::Json(v) => Bytes::from(v),
    })
}

async fn data_handler(
    uri: Uri,
    headers: HeaderMap,
    Query(params): Query<SeriesSelection>,
    State(state): State<AppState>,
) -> Result<Response> {
    serve(state, uri, headers, params, |q, r| {
        output_to_bytes(q.format(r)?)
    })
    .await
}

async fn data_bulk_handler(
    uri: Uri,
    headers: HeaderMap,
    Query(params): Query<SeriesSelection>,
    State(state): State<AppState>,
) -> Result<Response> {
    serve(state, uri, headers, params, |q, r| {
        output_to_bytes(q.format_bulk(r)?)
    })
    .await
}

async fn data_raw_handler(
    uri: Uri,
    headers: HeaderMap,
    Query(params): Query<SeriesSelection>,
    State(state): State<AppState>,
) -> Result<Response> {
    serve(state, uri, headers, params, |q, r| {
        output_to_bytes(q.format_raw(r)?)
    })
    .await
}

pub trait ApiSeriesRoutes {
    fn add_series_routes(self) -> Self;
}

impl ApiSeriesRoutes for ApiRouter<AppState> {
    fn add_series_routes(self) -> Self {
        self.api_route(
            "/api/series",
            get_with(
                async |uri: Uri, headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    state.respond_json(&headers, CacheStrategy::Deploy, &uri, |q| Ok(q.series_catalog())).await
                },
                |op| op
                    .id("get_series_tree")
                    .series_tag()
                    .summary("Series catalog")
                    .description(
                        "Returns the complete hierarchical catalog of available series organized as a tree structure. \
                        Series are grouped by categories and subcategories."
                    )
                    .json_response::<TreeNode>()
                    .not_modified(),
            ),
        )
        .api_route(
            "/api/series/count",
            get_with(
                async |
                    uri: Uri,
                    headers: HeaderMap,
                    _: Empty,
                    State(state): State<AppState>
                | {
                    state.respond_json(&headers, CacheStrategy::Deploy, &uri, |q| Ok(q.series_count())).await
                },
                |op| op
                    .id("get_series_count")
                    .series_tag()
                    .summary("Series count")
                    .description("Returns the number of series available per index type.")
                    .json_response::<DetailedSeriesCount>()
                    .not_modified(),
            ),
        )
        .api_route(
            "/api/series/indexes",
            get_with(
                async |
                    uri: Uri,
                    headers: HeaderMap,
                    _: Empty,
                    State(state): State<AppState>
                | {
                    state.respond_json(&headers, CacheStrategy::Deploy, &uri, |q| Ok(q.indexes())).await
                },
                |op| op
                    .id("get_indexes")
                    .series_tag()
                    .summary("List available indexes")
                    .description(
                        "Returns all available indexes with their accepted query aliases. Use any alias when querying series."
                    )
                    .json_response::<Vec<IndexInfo>>()
                    .not_modified(),
            ),
        )
        .api_route(
            "/api/series/list",
            get_with(
                async |
                    uri: Uri,
                    headers: HeaderMap,
                    State(state): State<AppState>,
                    Query(pagination): Query<Pagination>
                | {
                    state.respond_json(&headers, CacheStrategy::Deploy, &uri, move |q| Ok(q.series_list(pagination))).await
                },
                |op| op
                    .id("list_series")
                    .series_tag()
                    .summary("Series list")
                    .description("Paginated flat list of all available series names. Use `page` query param for pagination.")
                    .json_response::<PaginatedSeries>()
                    .not_modified(),
            ),
        )
        .api_route(
            "/api/series/search",
            get_with(
                async |
                    uri: Uri,
                    headers: HeaderMap,
                    State(state): State<AppState>,
                    Query(query): Query<SearchQuery>
                | {
                    state.respond_json(&headers, CacheStrategy::Deploy, &uri, move |q| Ok(q.search_series(&query))).await
                },
                |op| op
                    .id("search_series")
                    .series_tag()
                    .summary("Search series")
                    .description("Fuzzy search for series by name. Supports partial matches and typos.")
                    .json_response::<Vec<&str>>()
                    .not_modified()
                    .server_error(),
            ),
        )
        .api_route(
            "/api/series/{series}",
            get_with(
                async |
                    uri: Uri,
                    headers: HeaderMap,
                    _: Empty,
                    State(state): State<AppState>,
                    Path(path): Path<SeriesParam>
                | {
                    state.respond_json(&headers, CacheStrategy::Deploy, &uri, move |q| {
                        q.series_info(&path.series).ok_or_else(|| q.series_not_found_error(&path.series))
                    }).await
                },
                |op| op
                    .id("get_series_info")
                    .series_tag()
                    .summary("Get series info")
                    .description(
                        "Returns the supported indexes and value type for the specified series."
                    )
                    .json_response::<SeriesInfo>()
                    .not_modified()
                    .not_found()
                    .server_error(),
            ),
        )
        .api_route(
            "/api/series/{series}/{index}",
            get_with(
                async |uri: Uri,
                       headers: HeaderMap,
                       state: State<AppState>,
                       Path(path): Path<SeriesNameWithIndex>,
                       Query(range): Query<DataRangeFormat>|
                       -> Response {
                    data_handler(
                        uri,
                        headers,
                        Query(SeriesSelection::from((path.index, path.series, range))),
                        state,
                    )
                    .await
                    .into_response()
                },
                |op| op
                    .id("get_series")
                    .series_tag()
                    .summary("Get series data")
                    .description(
                        "Fetch data for a specific series at the given index. \
                        Use query parameters to filter by date range and format (json/csv)."
                    )
                    .json_response::<SeriesData>()
                    .csv_response()
                    .not_modified()
                    .not_found(),
            ),
        )
        .api_route(
            "/api/series/{series}/{index}/data",
            get_with(
                async |uri: Uri,
                       headers: HeaderMap,
                       state: State<AppState>,
                       Path(path): Path<SeriesNameWithIndex>,
                       Query(range): Query<DataRangeFormat>|
                       -> Response {
                    data_raw_handler(
                        uri,
                        headers,
                        Query(SeriesSelection::from((path.index, path.series, range))),
                        state,
                    )
                    .await
                    .into_response()
                },
                |op| op
                    .id("get_series_data")
                    .series_tag()
                    .summary("Get raw series data")
                    .description(
                        "Returns just the data array without the SeriesData wrapper. \
                        Supports the same range and format parameters as \
                        `GET /api/series/{series}/{index}`."
                    )
                    .json_response::<Vec<serde_json::Value>>()
                    .csv_response()
                    .not_modified()
                    .not_found(),
            ),
        )
        .api_route(
            "/api/series/{series}/{index}/latest",
            get_with(
                async |uri: Uri,
                       headers: HeaderMap,
                       _: Empty,
                       State(state): State<AppState>,
                       Path(path): Path<SeriesNameWithIndex>| {
                    state
                        .respond_json(&headers, CacheStrategy::Tip, &uri, move |q| {
                            q.latest(&path.series, path.index)
                        })
                        .await
                },
                |op| op
                    .id("get_series_latest")
                    .series_tag()
                    .summary("Get latest series value")
                    .description(
                        "Returns the single most recent value for a series, unwrapped (not inside a SeriesData object)."
                    )
                    .json_response::<serde_json::Value>()
                    .not_modified()
                    .not_found(),
            ),
        )
        .api_route(
            "/api/series/{series}/{index}/len",
            get_with(
                async |uri: Uri,
                       headers: HeaderMap,
                       _: Empty,
                       State(state): State<AppState>,
                       Path(path): Path<SeriesNameWithIndex>| {
                    state
                        .respond_json(&headers, CacheStrategy::Tip, &uri, move |q| {
                            q.len(&path.series, path.index)
                        })
                        .await
                },
                |op| op
                    .id("get_series_len")
                    .series_tag()
                    .summary("Get series data length")
                    .description("Returns the total number of data points for a series at the given index.")
                    .json_response::<usize>()
                    .not_modified()
                    .not_found(),
            ),
        )
        .api_route(
            "/api/series/{series}/{index}/version",
            get_with(
                async |uri: Uri,
                       headers: HeaderMap,
                       _: Empty,
                       State(state): State<AppState>,
                       Path(path): Path<SeriesNameWithIndex>| {
                    state
                        .respond_json(&headers, CacheStrategy::Tip, &uri, move |q| {
                            q.version(&path.series, path.index)
                        })
                        .await
                },
                |op| op
                    .id("get_series_version")
                    .series_tag()
                    .summary("Get series version")
                    .description("Returns the current version of a series. Changes when the series data is updated.")
                    .json_response::<Version>()
                    .not_modified()
                    .not_found(),
            ),
        )
        .api_route(
            "/api/series/bulk",
            get_with(
                |uri, headers, query, state| async move {
                    data_bulk_handler(uri, headers, query, state)
                        .await
                        .into_response()
                },
                |op| op
                    .id("get_series_bulk")
                    .series_tag()
                    .summary("Bulk series data")
                    .description(
                        "Fetch multiple series in a single request. Supports filtering by index and date range. \
                        Returns an array of SeriesData objects. For a single series, use `get_series` instead."
                    )
                    .json_response::<Vec<SeriesData>>()
                    .csv_response()
                    .not_modified(),
            ),
        )
    }
}
