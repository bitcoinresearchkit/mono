//! Live `/api/series/*` API: catalog, search, info, single-series, bulk.
//!
//! Holds the shared `serve` helper used by every series endpoint that returns
//! a formatted body (single, raw, and bulk).

use crate::request_state::RequestState;
use std::result::Result as StdResult;

use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    body::Bytes,
    extract::{Path, Query},
    http::HeaderMap,
    response::{IntoResponse, Response},
};
use bitview_catalog::TreeNode;
use bitview_query::{Output, Query as BrkQuery, ResolvedQuery, SeriesOutput};
use bitview_types::{
    DataRangeFormat, DetailedSeriesCount, Format, IndexInfo, PaginatedSeries, Pagination,
    SearchQuery, SeriesData, SeriesInfo, SeriesName, SeriesNameWithIndex, SeriesSelection,
};
use brk_error::Error as BrkError;
use brk_types::{Index, Version};
use serde_json::{Value, to_vec};

use crate::{
    AppState, CacheParams, CacheStrategy, CdnCacheMode, Error,
    error::Result,
    extended::{HeaderMapExtended, ResponseExtended, TransformResponseExtended},
    params::{Empty, SeriesParam},
};

pub fn serve_catalog(state: AppState, headers: HeaderMap) -> Response {
    state.series_bodies.catalog.respond(&headers)
}

pub fn serve_count(state: AppState, headers: HeaderMap) -> Response {
    state.series_bodies.count.respond(&headers)
}

pub fn serve_indexes(state: AppState, headers: HeaderMap) -> Response {
    state.series_bodies.indexes.respond(&headers)
}

pub async fn serve_list(state: AppState, headers: HeaderMap, pagination: Pagination) -> Response {
    state
        .respond_with_params(
            &headers,
            state.series_bodies.list.clone(),
            |headers| headers.insert_content_type_application_json(),
            move |query| Ok(Bytes::from(to_vec(&query.series_list(pagination))?)),
        )
        .await
}

pub async fn serve_search(state: AppState, headers: HeaderMap, search: SearchQuery) -> Response {
    if search.q.len() > MAX_INPUT_BYTES {
        return Error::bad_request(format!(
            "search query exceeds {MAX_INPUT_BYTES} UTF-8 bytes"
        ))
        .into_response();
    }
    if search.limit.is_zero() {
        return empty_search(&headers, state.cdn_cache_mode);
    }
    AppState::respond_with_future(&headers, state.series_bodies.search.clone(), async {
        let bytes = state
            .run_with_admission(&state.series_bodies.search_query, move |query| {
                Ok(Bytes::from(to_vec(&query.search_series(&search))?))
            })
            .await?;
        Ok((bytes, |headers: &mut HeaderMap| {
            headers.insert_content_type_application_json()
        }))
    })
    .await
}

const MAX_INPUT_BYTES: usize = 1024;

fn empty_search(headers: &HeaderMap, cdn_cache_mode: CdnCacheMode) -> Response {
    let params = CacheParams::resolve(&CacheStrategy::Immutable(Version::ONE), cdn_cache_mode);
    Response::json_bytes(headers, &params, || Bytes::from_static(b"[]"))
}

pub async fn serve_series_info(
    state: AppState,
    headers: HeaderMap,
    series: SeriesName,
) -> Result<Response> {
    validate_name(&series)?;
    if let Some(info) = state.sync(|q| q.resolve_series_info(&series)) {
        return Ok(Response::json_bytes(
            &headers,
            &state.series_bodies.info,
            || Bytes::from(to_vec(&info.into_info()).unwrap()),
        ));
    }

    let error = state
        .read_with_admission(&state.series_bodies.search_query, move |q| {
            Ok(q.missing_series_error(&series))
        })
        .await?;
    Err(error.into())
}

fn validate_name(series: &SeriesName) -> Result<()> {
    if series.len() > MAX_INPUT_BYTES {
        return Err(Error::bad_request(format!(
            "series name exceeds {MAX_INPUT_BYTES} UTF-8 bytes"
        )));
    }
    Ok(())
}

pub async fn serve_latest(
    state: AppState,
    headers: HeaderMap,
    series: SeriesName,
    index: Index,
) -> Result<Response> {
    validate_name(&series)?;
    let bytes = state
        .read_with_admission(&state.series_bodies.data_query, move |q| {
            Ok(Bytes::from(q.latest_json(&series, index)?))
        })
        .await?;
    Ok(state.respond_json_content_bytes(&headers, bytes))
}

pub async fn serve_len(
    state: AppState,
    headers: HeaderMap,
    series: SeriesName,
    index: Index,
) -> Result<Response> {
    validate_name(&series)?;
    let length = state
        .read_with_admission(&state.series_bodies.data_query, move |q| {
            q.len(&series, index)
        })
        .await?;
    // The JSON number depends only on length, not on the chain tip or deploy.
    let strategy = CacheStrategy::Live(format!("len1-{length}").into());
    Ok(state.respond_json_value(&headers, strategy, length))
}

pub async fn serve_version(
    state: AppState,
    headers: HeaderMap,
    series: SeriesName,
    index: Index,
) -> Result<Response> {
    validate_name(&series)?;
    if let Some(version) = state.sync(|q| q.find_version(&series, index))? {
        let strategy = CacheStrategy::Live(format!("sv1-{version}").into());
        return Ok(state.respond_json_value(&headers, strategy, version));
    }
    let error = state
        .read_with_admission(&state.series_bodies.search_query, move |q| {
            Ok(q.missing_series_error(&series))
        })
        .await?;
    Err(error.into())
}

/// Shared response pipeline for every series endpoint.
///
/// One admitted blocking job keeps resolution and formatting under the same
/// publication guard. Matching validators skip filename and body preparation.
pub async fn serve(
    state: AppState,
    headers: HeaderMap,
    params: SeriesSelection,
    to_bytes: impl FnOnce(&BrkQuery, ResolvedQuery) -> StdResult<Bytes, BrkError>
    + Clone
    + Send
    + 'static,
) -> Result<Response> {
    params.series.iter().try_for_each(validate_name)?;
    let max_weight = state.max_weight;
    let cdn_cache_mode = state.cdn_cache_mode;
    state
        .read_with_admission(&state.series_bodies.data_query, move |q| {
            let resolved = q.resolve(params, max_weight)?;
            let cache_params = CacheParams::series(
                resolved.version,
                resolved.start,
                resolved.end,
                resolved.stable_count,
                resolved.hash_prefix,
                cdn_cache_mode,
            );
            if cache_params.matches_etag(&headers) {
                return Ok(Response::new_not_modified(&cache_params));
            }
            let csv_filename = match resolved.format {
                Format::CSV => Some(resolved.csv_filename()),
                Format::JSON => None,
            };
            let bytes = to_bytes(q, resolved)?;
            Ok(AppState::assemble_response(
                cache_params,
                bytes,
                move |h| match csv_filename {
                    Some(filename) => {
                        h.insert_content_disposition_attachment(&filename);
                        h.insert_content_type_text_csv();
                    }
                    None => h.insert_content_type_application_json(),
                },
            ))
        })
        .await
        .map_err(Into::into)
}

fn output_to_bytes(out: SeriesOutput) -> Bytes {
    match out.output {
        Output::CSV(s) => Bytes::from(s),
        Output::Json(v) => Bytes::from(v),
    }
}

async fn data_handler(
    headers: HeaderMap,
    Query(params): Query<SeriesSelection>,
    RequestState(state): RequestState,
) -> Result<Response> {
    serve(state, headers, params, |q, r| {
        q.format(r).map(output_to_bytes)
    })
    .await
}

async fn data_bulk_handler(
    headers: HeaderMap,
    Query(params): Query<SeriesSelection>,
    RequestState(state): RequestState,
) -> Result<Response> {
    serve(state, headers, params, |q, r| {
        q.format_bulk(r).map(output_to_bytes)
    })
    .await
}

async fn data_raw_handler(
    headers: HeaderMap,
    Query(params): Query<SeriesSelection>,
    RequestState(state): RequestState,
) -> Result<Response> {
    serve(state, headers, params, |q, r| {
        q.format_raw(r).map(output_to_bytes)
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
                async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                    serve_catalog(state, headers)
                },
                |op| op
                    .id("get_series_tree")
                    .series_tag()
                    .mcp_ignore()
                    .summary("Series catalog")
                    .description(
                        "Returns the complete hierarchical catalog of available series organized as a tree structure. \
                        Series are grouped by categories and subcategories."
                    )
                    .json_response::<TreeNode>()
                    .bad_request()
                    .not_modified(),
            ),
        )
        .api_route(
            "/api/series/count",
            get_with(
                async |
                    headers: HeaderMap,
                    _: Empty,
                    RequestState(state): RequestState
                | {
                    serve_count(state, headers)
                },
                |op| op
                    .id("get_series_count")
                    .series_tag()
                    .summary("Series count")
                    .description("Returns the number of series available per index type.")
                    .json_response::<DetailedSeriesCount>()
                    .bad_request()
                    .not_modified(),
            ),
        )
        .api_route(
            "/api/series/indexes",
            get_with(
                async |
                    headers: HeaderMap,
                    _: Empty,
                    RequestState(state): RequestState
                | {
                    serve_indexes(state, headers)
                },
                |op| op
                    .id("get_indexes")
                    .series_tag()
                    .summary("List available indexes")
                    .description(
                        "Returns all available indexes with their accepted query aliases. Use any alias when querying series."
                    )
                    .json_response::<Vec<IndexInfo>>()
                    .bad_request()
                    .not_modified(),
            ),
        )
        .api_route(
            "/api/series/list",
            get_with(
                async |
                    headers: HeaderMap,
                    RequestState(state): RequestState,
                    Query(pagination): Query<Pagination>
                | {
                    serve_list(state, headers, pagination).await
                },
                |op| op
                    .id("list_series")
                    .series_tag()
                    .summary("Series list")
                    .server_error()
                    .gateway_timeout()
                    .description("Paginated flat list of all available series names. Use `page` query param for pagination.")
                    .json_response::<PaginatedSeries>()
                    .bad_request()
                    .not_modified(),
            ),
        )
        .api_route(
            "/api/series/search",
            get_with(
                async |
                    headers: HeaderMap,
                    RequestState(state): RequestState,
                    Query(query): Query<SearchQuery>
                | {
                    serve_search(state, headers, query).await
                },
                |op| op
                    .id("search_series")
                    .series_tag()
                    .summary("Search series")
                    .gateway_timeout()
                    .description(&format!("Search series by name or descriptive terms. Results prioritize whole query words in names, then descriptions, then fuzzy names, then fuzzy descriptions. Word order does not matter. Descriptions provide cohort terminology and formulas. The decoded q parameter is limited to {MAX_INPUT_BYTES} UTF-8 bytes."))
                    .json_response::<Vec<&str>>()
                    .bad_request()
                    .not_modified()
                    .gateway_timeout()
                    .server_error(),
            ),
        )
        .api_route(
            "/api/series/{series}",
            get_with(
                async |
                    headers: HeaderMap,
                    _: Empty,
                    RequestState(state): RequestState,
                    Path(path): Path<SeriesParam>
                | -> Result<Response> {
                    serve_series_info(state, headers, path.series).await
                },
                |op| op
                    .id("get_series_info")
                    .series_tag()
                    .summary("Get series info")
                    .gateway_timeout()
                    .description(&format!("Returns the optional description, supported indexes, and value type for the specified series. The decoded series name is limited to {MAX_INPUT_BYTES} UTF-8 bytes."))
                    .json_response::<SeriesInfo>()
                    .bad_request()
                    .gateway_timeout()
                    .not_modified()
                    .not_found()
                    .server_error(),
            ),
        )
        .api_route(
            "/api/series/{series}/{index}",
            get_with(
                async |headers: HeaderMap,
                       state: RequestState,
                       Path(path): Path<SeriesNameWithIndex>,
                       Query(range): Query<DataRangeFormat>|
                       -> Response {
                    data_handler(
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
                    .server_errors()
                    .description(
                        "Fetch data for a specific series at the given index. \
                        Use query parameters to filter by date range and format (json/csv)."
                    )
                    .json_response::<SeriesData>()
                    .csv_response()
                    .bad_request()
                    .not_modified()
                    .not_found(),
            ),
        )
        .api_route(
            "/api/series/{series}/{index}/data",
            get_with(
                async |headers: HeaderMap,
                       state: RequestState,
                       Path(path): Path<SeriesNameWithIndex>,
                       Query(range): Query<DataRangeFormat>|
                       -> Response {
                    data_raw_handler(
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
                    .mcp_ignore()
                    .summary("Get raw series data")
                    .server_errors()
                    .description(
                        "Returns just the data array without the SeriesData wrapper. \
                        Supports the same range and format parameters as \
                        `GET /api/series/{series}/{index}`."
                    )
                    .json_response::<Vec<Value>>()
                    .csv_response()
                    .bad_request()
                    .not_modified()
                    .not_found(),
            ),
        )
        .api_route(
            "/api/series/{series}/{index}/latest",
            get_with(
                async |headers: HeaderMap,
                       _: Empty,
                       RequestState(state): RequestState,
                       Path(path): Path<SeriesNameWithIndex>| {
                    serve_latest(state, headers, path.series, path.index).await
                },
                |op| op
                    .id("get_series_latest")
                    .series_tag()
                    .summary("Get latest series value")
                    .server_errors()
                    .description(
                        "Returns the single most recent value for a series, unwrapped (not inside a SeriesData object)."
                    )
                    .json_response::<Value>()
                    .bad_request()
                    .not_modified()
                    .not_found(),
            ),
        )
        .api_route(
            "/api/series/{series}/{index}/len",
            get_with(
                async |headers: HeaderMap,
                       _: Empty,
                       RequestState(state): RequestState,
                       Path(path): Path<SeriesNameWithIndex>| {
                    serve_len(state, headers, path.series, path.index).await
                },
                |op| op
                    .id("get_series_len")
                    .series_tag()
                    .summary("Get series data length")
                    .server_errors()
                    .description("Returns the total number of data points for a series at the given index.")
                    .json_response::<usize>()
                    .bad_request()
                    .not_modified()
                    .not_found(),
            ),
        )
        .api_route(
            "/api/series/{series}/{index}/version",
            get_with(
                async |headers: HeaderMap,
                       _: Empty,
                       RequestState(state): RequestState,
                       Path(path): Path<SeriesNameWithIndex>|
                       -> Result<Response> {
                    serve_version(state, headers, path.series, path.index).await
                },
                |op| op
                    .id("get_series_version")
                    .series_tag()
                    .summary("Get series version")
                    .server_error()
                    .gateway_timeout()
                    .bad_request()
                    .description("Returns the vector's schema/computation version, not its length or latest update. Appends and reorgs do not by themselves change this version.")
                    .json_response::<Version>()
                    .not_modified()
                    .not_found(),
            ),
        )
        .api_route(
            "/api/series/bulk",
            get_with(
                |headers, query, state| async move {
                    data_bulk_handler(headers, query, state)
                        .await
                        .into_response()
                },
                |op| op
                    .id("get_series_bulk")
                    .series_tag()
                    .summary("Bulk series data")
                    .server_errors()
                    .not_found()
                    .description(
                        "Fetch multiple series in a single request. Supports filtering by index and date range. \
                        Returns an array of SeriesData objects. For a single series, use `get_series` instead."
                    )
                    .json_response::<Vec<SeriesData>>()
                    .csv_response()
                    .bad_request()
                    .not_modified(),
            ),
        )
    }
}
