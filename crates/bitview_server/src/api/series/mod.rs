//! Live `/api/series/*` API: catalog, search, info, single-series, bulk.

use std::{fmt::Write, result::Result as StdResult, slice};

use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    body::Bytes,
    extract::{Path, Query},
    http::{HeaderMap, HeaderValue, header},
    response::{IntoResponse, Response},
};
use bitview_catalog::TreeNode;
use bitview_primitives::{Index, SeriesName};
use bitview_query::{Error as QueryError, Output, Query as BrkQuery, ResolvedQuery, SeriesAnchor};
use bitview_types::{
    DataRangeFormat, DetailedSeriesCount, Format, PaginatedSeries, SearchQuery, SeriesData,
    SeriesInfo, SeriesListQuery, SeriesNameWithIndex, SeriesSelection,
};
use brk_types::Version;
use serde_json::Value;

use crate::{
    AppState, Error,
    answer::{Answer, Content, JSON},
    api::tags,
    error::Result,
    extended::TransformResponseExtended,
    params::{Empty, SeriesParam},
    request_state::RequestState,
};

/// A page of the names, at most a thousand of them, from the registry in memory: settled, as only
/// a release changes it.
pub fn serve_list(state: AppState, headers: &HeaderMap, query: SeriesListQuery) -> Response {
    if query.q.len() > MAX_INPUT_BYTES {
        return Error::bad_request(format!("list filter exceeds {MAX_INPUT_BYTES} UTF-8 bytes"))
            .into_response();
    }
    let tag = state.series_bodies.list.clone();
    state.respond_now(
        headers,
        Answer::json(tag, move |q| Ok(q.series_list(&query))).settled(true),
    )
}

pub async fn serve_search(state: AppState, headers: &HeaderMap, search: SearchQuery) -> Response {
    if search.q.len() > MAX_INPUT_BYTES {
        return Error::bad_request(format!(
            "search query exceeds {MAX_INPUT_BYTES} UTF-8 bytes"
        ))
        .into_response();
    }
    let tag = state.series_bodies.search.clone();
    state
        .respond_named(headers, state.series_search(), tag, move |q| {
            Content::json(&q.search_series(&search))
        })
        .await
}

const MAX_INPUT_BYTES: usize = 1024;

pub async fn serve_series_info(
    state: AppState,
    headers: &HeaderMap,
    series: SeriesName,
) -> Result<Response> {
    validate_name(&series)?;
    let Some(info) = state.query.sync(|q| q.resolve_series_info(&series)) else {
        return Err(missing_series(&state, &series));
    };
    // Registry facts only, so only a release changes them.
    let tag = state.series_bodies.info.clone();
    Ok(state.respond_now(headers, Answer::json(tag, move |_| Ok(info.into_info())).settled(true)))
}

fn validate_name(series: &SeriesName) -> Result<()> {
    if series.len() > MAX_INPUT_BYTES {
        return Err(Error::bad_request(format!(
            "series name exceeds {MAX_INPUT_BYTES} UTF-8 bytes"
        )));
    }
    Ok(())
}

/// The error for a name no series has, with its nearest names: read from the names alone, so it
/// waits behind nothing.
fn missing_series(state: &AppState, series: &SeriesName) -> Error {
    state.query.sync(|q| q.missing_series_error(series)).into()
}

/// A malformed name anywhere in a bulk request takes precedence over lookup errors.
fn check_series(state: &AppState, names: &[SeriesName], index: Index) -> Result<()> {
    names.iter().try_for_each(validate_name)?;
    state.query.sync(|q| names.iter().try_for_each(|name| q.check_series(name, index)))?;
    Ok(())
}

pub async fn serve_latest(
    state: AppState,
    headers: &HeaderMap,
    series: SeriesName,
    index: Index,
) -> Result<Response> {
    check_series(&state, slice::from_ref(&series), index)?;
    Ok(state
        .respond(headers, state.series_data(), move |q| {
            Ok(Answer::built(Content::new(JSON, q.latest_json(&series, index)?)))
        })
        .await)
}

pub async fn serve_len(
    state: AppState,
    headers: &HeaderMap,
    series: SeriesName,
    index: Index,
) -> Result<Response> {
    check_series(&state, slice::from_ref(&series), index)?;
    Ok(state
        .respond(headers, state.series_data(), move |q| {
            let length = q.len(&series, index)?;
            // The JSON number depends only on the length, not on the chain tip.
            Ok(Answer::ready(format!("len-{length}"), Content::json(&length)?))
        })
        .await)
}

pub async fn serve_version(
    state: AppState,
    headers: &HeaderMap,
    series: SeriesName,
    index: Index,
) -> Result<Response> {
    validate_name(&series)?;
    let Some(version) = state.query.sync(|q| q.find_version(&series, index))? else {
        return Err(missing_series(&state, &series));
    };
    let answer = Answer::ready(format!("sv-{version}"), Content::json(&version)?);
    Ok(state.respond_now(headers, answer))
}

/// What names resolved rows, and whether they are settled:
/// - historical: `s{v}-r{start}-{end}-{anchor:x}`, the anchor being the last block the rows
///   depend on (for calendar indexes, the following period's first block): stable across
///   appends, changed by any reorg replacing those rows. Settled.
/// - tip: `s{v}-t{tip_hash:x}`. Changes per block, reorg-safe.
fn series_tag(query: &ResolvedQuery) -> (String, bool) {
    let v = u32::from(query.version());
    let range = query.range();
    match query.anchor() {
        SeriesAnchor::Historical(hash) => (
            format!("s{v}-r{}-{}-{:x}", range.start, range.end, *hash),
            true,
        ),
        SeriesAnchor::Tip(hash) => (format!("s{v}-t{:x}", *hash), false),
    }
}

/// `Content-Disposition` for a download named `filename`.
fn attachment(filename: &str) -> HeaderValue {
    if filename.is_ascii() && !filename.contains('"') && !filename.contains('\\') {
        let quoted = format!("attachment; filename=\"{filename}\"");
        if let Ok(value) = HeaderValue::try_from(quoted) {
            return value;
        }
    }
    let mut value = String::from("attachment; filename*=UTF-8''");
    for byte in filename.bytes() {
        write!(value, "%{byte:02X}").unwrap();
    }
    HeaderValue::try_from(value).expect("ASCII attachment header")
}

/// Shared response pipeline for formatted series data.
///
/// One admitted blocking job keeps resolution and formatting under the same
/// pin. Matching validators skip filename and body preparation.
pub async fn serve(
    state: AppState,
    headers: &HeaderMap,
    params: SeriesSelection,
    to_bytes: impl FnOnce(&BrkQuery, ResolvedQuery) -> StdResult<Bytes, QueryError>
    + Clone
    + Send
    + 'static,
) -> Result<Response> {
    check_series(&state, &params.series, params.index)?;
    let max_weight = state.max_weight;
    Ok(state
        .respond(headers, state.series_data(), move |q| {
            let resolved = q.resolve(params, max_weight)?;
            let (tag, settled) = series_tag(&resolved);
            let answer = Answer::new(tag, move |q| {
                Ok(match resolved.format() {
                    Format::CSV => {
                        let filename = attachment(&resolved.csv_filename());
                        Content::new("text/csv", to_bytes(q, resolved)?)
                            .with(header::CONTENT_DISPOSITION, filename)
                    }
                    Format::JSON => Content::new(JSON, to_bytes(q, resolved)?),
                })
            });
            Ok(answer.settled(settled))
        })
        .await)
}

fn output_to_bytes(out: Output) -> Bytes {
    match out {
        Output::CSV(s) => Bytes::from(s),
        Output::Json(v) => Bytes::from(v),
    }
}

pub(super) fn add_routes(router: ApiRouter<AppState>) -> ApiRouter<AppState> {
    router.api_route(
        "/api/series",
        get_with(
            async |headers: HeaderMap, _: Empty, RequestState(state): RequestState| {
                state.respond_now(&headers, state.series_bodies.catalog.answer())
            },
            |op| op
                .id("get_series_tree")
                .tag(tags::SERIES)
                .mcp_ignore()
                .summary("Series catalog")
                .description(
                    "The available series as a tree of categories and subcategories."
                )
                .json_response::<TreeNode>()
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
                state.respond_now(&headers, state.series_bodies.count.answer())
            },
            |op| op
                .id("get_series_count")
                .tag(tags::SERIES)
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
                headers: HeaderMap,
                _: Empty,
                RequestState(state): RequestState
            | {
                state.respond_now(&headers, state.series_bodies.indexes.answer())
            },
            |op| op
                .id("list_indexes")
                .tag(tags::SERIES)
                .summary("List available indexes")
                .description(
                    "Returns every index a series can be read at, by the name series routes take."
                )
                .json_response::<Vec<Index>>()
                .not_modified(),
        ),
    )
    .api_route(
        "/api/series/list",
        get_with(
            async |
                headers: HeaderMap,
                RequestState(state): RequestState,
                Query(query): Query<SeriesListQuery>
            | {
                serve_list(state, &headers, query)
            },
            |op| op
                .id("list_series")
                .tag(tags::SERIES)
                .summary("Series list")
                .description(&format!("Paginated flat list of the available series names: all of them, or with `q` only those that contain that text as written (a plain match, unlike `search_series`; at most {MAX_INPUT_BYTES} UTF-8 bytes)."))
                .json_response::<PaginatedSeries>()
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
                serve_search(state, &headers, query).await
            },
            |op| op
                .id("search_series")
                .tag(tags::SERIES)
                .summary("Search series")
                .description(&format!("Find on-chain metrics by plain words: MVRV, realized price, SOPR, NUPL, supply held by long-term or short-term holders, coin age, addresses by balance, hashrate, fees, volume, and thousands more. Start here for any metric question: it returns the series ids to pass to `get_series_latest` or `get_series`. Results prioritize whole query words in names, then names matching every word by its start, joined words or a plural, then descriptions holding every word whole, singular or plural (never a part of a word or a typo), then names with a typo. Word order does not matter. Descriptions provide cohort terminology and formulas. The decoded q parameter is limited to {MAX_INPUT_BYTES} UTF-8 bytes."))
                .json_response::<Vec<SeriesName>>()
                .not_modified(),
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
                serve_series_info(state, &headers, path.series).await
            },
            |op| op
                .id("get_series_info")
                .tag(tags::SERIES)
                .summary("Get series info")
                .description(&format!("Returns the optional description, supported indexes, the indexes whose values can be null, the value type, the optional unit (what the value type measures), and the accumulation (how its values combine over time) of the specified series. The decoded series name is limited to {MAX_INPUT_BYTES} UTF-8 bytes."))
                .json_response::<SeriesInfo>()
                .not_modified()
                .not_found(),
        ),
    )
    .api_route(
        "/api/series/{series}/{index}",
        get_with(
            async |headers: HeaderMap,
                   RequestState(state): RequestState,
                   Path(path): Path<SeriesNameWithIndex>,
                   Query(range): Query<DataRangeFormat>|
                   -> Response {
                let params = SeriesSelection::from((path.index, path.series, range));
                serve(state, &headers, params, |q, r| q.format(r).map(output_to_bytes))
                    .await
                    .into_response()
            },
            |op| op
                .id("get_series")
                .tag(tags::SERIES)
                .summary("Get series data")
                .description(
                    "History of an on-chain metric, by the series id from `search_series`: rows selected by a range of positions, dates or timestamps, as JSON or CSV."
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
            async |headers: HeaderMap,
                   RequestState(state): RequestState,
                   Path(path): Path<SeriesNameWithIndex>,
                   Query(range): Query<DataRangeFormat>|
                   -> Response {
                let params = SeriesSelection::from((path.index, path.series, range));
                serve(state, &headers, params, |q, r| q.format_raw(r).map(output_to_bytes))
                    .await
                    .into_response()
            },
            |op| op
                .id("get_series_data")
                .tag(tags::SERIES)
                .mcp_ignore()
                .summary("Get raw series data")
                .description(
                    "The data array without the `SeriesData` wrapper. \
                    Supports the same range and format parameters as \
                    `GET /api/series/{series}/{index}`."
                )
                .json_response::<Vec<Value>>()
                .csv_response()
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
                serve_latest(state, &headers, path.series, path.index).await
            },
            |op| op
                .id("get_series_latest")
                .tag(tags::SERIES)
                .summary("Get latest series value")
                .description(
                    "Current value of an on-chain metric, by the series id from `search_series`. Returns the value alone, not inside a SeriesData object."
                )
                .json_response::<Value>()
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
                serve_len(state, &headers, path.series, path.index).await
            },
            |op| op
                .id("get_series_len")
                .tag(tags::SERIES)
                .summary("Get series data length")
                .json_response::<usize>()
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
                serve_version(state, &headers, path.series, path.index).await
            },
            |op| op
                .id("get_series_version")
                .tag(tags::SERIES)
                .summary("Get series version")
                .description("Returns the vector's schema/computation version, not its length or latest update. Appends and reorgs do not by themselves change this version.")
                .json_response::<Version>()
                .not_modified()
                .not_found(),
        ),
    )
    .api_route(
        "/api/series/bulk",
        get_with(
            async |headers: HeaderMap,
                   Query(params): Query<SeriesSelection>,
                   RequestState(state): RequestState|
                   -> Response {
                serve(state, &headers, params, |q, r| q.format_bulk(r).map(output_to_bytes))
                    .await
                    .into_response()
            },
            |op| op
                .id("get_series_bulk")
                .tag(tags::SERIES)
                .summary("Bulk series data")
                .not_found()
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
