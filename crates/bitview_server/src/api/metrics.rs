use aide::axum::{ApiRouter, routing::get_with};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, Uri},
    response::{IntoResponse, Response},
};
use bitview_traversable::TreeNode;
use bitview_types::{
    DataRangeFormat, DetailedSeriesCount, IndexInfo, PaginatedSeries, Pagination, SearchQuery,
    SeriesData, SeriesInfo, SeriesList, SeriesName, SeriesSelection, SeriesSelectionLegacy,
};
use brk_types::Index;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{CacheStrategy, Error, extended::TransformResponseExtended, params::Empty};

use super::AppState;
use super::series_legacy;

/// Legacy path parameter for `/api/metric/{metric}`
#[derive(Deserialize, JsonSchema)]
struct LegacySeriesParam {
    metric: SeriesName,
}

/// Legacy path parameters for `/api/metric/{metric}/{index}`
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
struct LegacySeriesWithIndex {
    metric: SeriesName,
    index: Index,
}

pub trait ApiMetricsLegacyRoutes {
    fn add_metrics_legacy_routes(self) -> Self;
}

impl ApiMetricsLegacyRoutes for ApiRouter<AppState> {
    fn add_metrics_legacy_routes(self) -> Self {
        self
        // --- Deprecated /api/metrics routes ---
        .api_route(
            "/api/metrics",
            get_with(
                async |uri: Uri, headers: HeaderMap, _: Empty, State(state): State<AppState>| {
                    state.respond_json(&headers, CacheStrategy::Deploy, &uri, |q| Ok(q.series_catalog())).await
                },
                |op| op
                    .id("get_metrics_tree_deprecated")
                    .metrics_tag()
                    .deprecated()
                    .summary("Metrics catalog (deprecated)")
                    .description(
                        "**DEPRECATED** - Use `/api/series` instead.\n\n\
                        Sunset date: 2027-01-01."
                    )
                    .json_response::<TreeNode>()
                    .not_modified(),
            ),
        )
        .api_route(
            "/api/metrics/count",
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
                    .id("get_metrics_count_deprecated")
                    .metrics_tag()
                    .deprecated()
                    .summary("Metric count (deprecated)")
                    .description(
                        "**DEPRECATED** - Use `/api/series/count` instead.\n\n\
                        Sunset date: 2027-01-01."
                    )
                    .json_response::<DetailedSeriesCount>()
                    .not_modified(),
            ),
        )
        .api_route(
            "/api/metrics/indexes",
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
                    .id("get_indexes_deprecated")
                    .metrics_tag()
                    .deprecated()
                    .summary("List available indexes (deprecated)")
                    .description(
                        "**DEPRECATED** - Use `/api/series/indexes` instead.\n\n\
                        Sunset date: 2027-01-01."
                    )
                    .json_response::<Vec<IndexInfo>>()
                    .not_modified(),
            ),
        )
        .api_route(
            "/api/metrics/list",
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
                    .id("list_metrics_deprecated")
                    .metrics_tag()
                    .deprecated()
                    .summary("Metrics list (deprecated)")
                    .description(
                        "**DEPRECATED** - Use `/api/series/list` instead.\n\n\
                        Sunset date: 2027-01-01."
                    )
                    .json_response::<PaginatedSeries>()
                    .not_modified(),
            ),
        )
        .api_route(
            "/api/metrics/search",
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
                    .id("search_metrics_deprecated")
                    .metrics_tag()
                    .deprecated()
                    .summary("Search metrics (deprecated)")
                    .description(
                        "**DEPRECATED** - Use `/api/series/search` instead.\n\n\
                        Sunset date: 2027-01-01."
                    )
                    .json_response::<Vec<&str>>()
                    .not_modified()
                    .server_error(),
            ),
        )
        .api_route(
            "/api/metrics/bulk",
            get_with(
                |uri: Uri, headers: HeaderMap, query: Query<SeriesSelection>, state: State<AppState>| async move {
                    series_legacy::handler(uri, headers, query, state)
                        .await
                        .into_response()
                },
                |op| op
                    .id("get_metrics_bulk_deprecated")
                    .metrics_tag()
                    .deprecated()
                    .summary("Bulk metric data (deprecated)")
                    .description(
                        "**DEPRECATED** - Use `/api/series/bulk` instead.\n\n\
                        Sunset date: 2027-01-01."
                    )
                    .json_response::<Vec<SeriesData>>()
                    .csv_response()
                    .not_modified(),
            ),
        )
        // --- Deprecated /api/metric/{metric} routes ---
        .api_route(
            "/api/metric/{metric}",
            get_with(
                async |
                    uri: Uri,
                    headers: HeaderMap,
                    _: Empty,
                    State(state): State<AppState>,
                    Path(path): Path<LegacySeriesParam>
                | {
                    state.respond_json(&headers, CacheStrategy::Deploy, &uri, move |q| {
                        q.series_info(&path.metric).ok_or_else(|| q.series_not_found_error(&path.metric))
                    }).await
                },
                |op| op
                    .id("get_metric_info_deprecated")
                    .metrics_tag()
                    .deprecated()
                    .summary("Get metric info (deprecated)")
                    .description(
                        "**DEPRECATED** - Use `/api/series/{series}` instead.\n\n\
                        Sunset date: 2027-01-01."
                    )
                    .json_response::<SeriesInfo>()
                    .not_modified()
                    .not_found()
                    .server_error(),
            ),
        )
        .api_route(
            "/api/metric/{metric}/{index}",
            get_with(
                async |uri: Uri,
                       headers: HeaderMap,
                       state: State<AppState>,
                       Path(path): Path<LegacySeriesWithIndex>,
                       Query(range): Query<DataRangeFormat>|
                       -> Response {
                    let params = SeriesSelection::from((path.index, path.metric, range));
                    series_legacy::handler(uri, headers, Query(params), state)
                        .await
                        .into_response()
                },
                |op| op
                    .id("get_metric_deprecated")
                    .metrics_tag()
                    .deprecated()
                    .summary("Get metric data (deprecated)")
                    .description(
                        "**DEPRECATED** - Use `/api/series/{series}/{index}` instead.\n\n\
                        Sunset date: 2027-01-01."
                    )
                    .json_response::<SeriesData>()
                    .csv_response()
                    .not_modified()
                    .not_found(),
            ),
        )
        .api_route(
            "/api/metric/{metric}/{index}/data",
            get_with(
                async |uri: Uri,
                       headers: HeaderMap,
                       state: State<AppState>,
                       Path(path): Path<LegacySeriesWithIndex>,
                       Query(range): Query<DataRangeFormat>|
                       -> Response {
                    let params = SeriesSelection::from((path.index, path.metric, range));
                    series_legacy::handler(uri, headers, Query(params), state)
                        .await
                        .into_response()
                },
                |op| op
                    .id("get_metric_data_deprecated")
                    .metrics_tag()
                    .deprecated()
                    .summary("Get raw metric data (deprecated)")
                    .description(
                        "**DEPRECATED** - Use `/api/series/{series}/{index}/data` instead.\n\n\
                        Sunset date: 2027-01-01."
                    )
                    .json_response::<Vec<serde_json::Value>>()
                    .csv_response()
                    .not_modified()
                    .not_found(),
            ),
        )
        .api_route(
            "/api/metric/{metric}/{index}/latest",
            get_with(
                async |uri: Uri,
                       headers: HeaderMap,
                       _: Empty,
                       State(state): State<AppState>,
                       Path(path): Path<LegacySeriesWithIndex>| {
                    state
                        .respond_json(&headers, CacheStrategy::Tip, &uri, move |q| {
                            q.latest(&path.metric, path.index)
                        })
                        .await
                },
                |op| op
                    .id("get_metric_latest_deprecated")
                    .metrics_tag()
                    .deprecated()
                    .summary("Get latest metric value (deprecated)")
                    .description(
                        "**DEPRECATED** - Use `/api/series/{series}/{index}/latest` instead.\n\n\
                        Sunset date: 2027-01-01."
                    )
                    .json_response::<serde_json::Value>()
                    .not_found(),
            ),
        )
        .api_route(
            "/api/metric/{metric}/{index}/len",
            get_with(
                async |uri: Uri,
                       headers: HeaderMap,
                       _: Empty,
                       State(state): State<AppState>,
                       Path(path): Path<LegacySeriesWithIndex>| {
                    state
                        .respond_json(&headers, CacheStrategy::Tip, &uri, move |q| {
                            q.len(&path.metric, path.index)
                        })
                        .await
                },
                |op| op
                    .id("get_metric_len_deprecated")
                    .metrics_tag()
                    .deprecated()
                    .summary("Get metric data length (deprecated)")
                    .description(
                        "**DEPRECATED** - Use `/api/series/{series}/{index}/len` instead.\n\n\
                        Sunset date: 2027-01-01."
                    )
                    .json_response::<usize>()
                    .not_found(),
            ),
        )
        .api_route(
            "/api/metric/{metric}/{index}/version",
            get_with(
                async |uri: Uri,
                       headers: HeaderMap,
                       _: Empty,
                       State(state): State<AppState>,
                       Path(path): Path<LegacySeriesWithIndex>| {
                    state
                        .respond_json(&headers, CacheStrategy::Tip, &uri, move |q| {
                            q.version(&path.metric, path.index)
                        })
                        .await
                },
                |op| op
                    .id("get_metric_version_deprecated")
                    .metrics_tag()
                    .deprecated()
                    .summary("Get metric version (deprecated)")
                    .description(
                        "**DEPRECATED** - Use `/api/series/{series}/{index}/version` instead.\n\n\
                        Sunset date: 2027-01-01."
                    )
                    .json_response::<brk_types::Version>()
                    .not_found(),
            ),
        )
        // --- Deprecated /api/vecs/ routes (moved from series module) ---
        .api_route(
            "/api/vecs/{variant}",
            get_with(
                async |uri: Uri,
                       headers: HeaderMap,
                       Path(variant): Path<String>,
                       Query(range): Query<DataRangeFormat>,
                       state: State<AppState>|
                       -> Response {
                    let separator = "_to_";
                    let variant = variant.replace("-", "_");
                    let mut split = variant.split(separator);

                    let ser_index = split.next().unwrap();
                    let Ok(index) = Index::try_from(ser_index) else {
                        return Error::not_found(
                            format!("Index '{ser_index}' doesn't exist")
                        ).into_response();
                    };

                    let params = SeriesSelection::from((
                        index,
                        SeriesList::from(split.collect::<Vec<_>>().join(separator)),
                        range,
                    ));
                    series_legacy::handler(uri, headers, Query(params), state)
                        .await
                        .into_response()
                },
                |op| op
                    .metrics_tag()
                    .summary("Legacy variant endpoint")
                    .description(
                        "**DEPRECATED** - Use `/api/series/{series}/{index}` instead.\n\n\
                        Sunset date: 2027-01-01. May be removed earlier in case of abuse.\n\n\
                        Legacy endpoint for querying series by variant path (e.g., `day1_to_price`). \
                        Returns raw data without the SeriesData wrapper."
                    )
                    .deprecated()
                    .json_response::<serde_json::Value>()
                    .not_modified(),
            ),
        )
        .api_route(
            "/api/vecs/query",
            get_with(
                async |uri: Uri,
                       headers: HeaderMap,
                       Query(params): Query<SeriesSelectionLegacy>,
                       state: State<AppState>|
                       -> Response {
                    let params: SeriesSelection = params.into();
                    series_legacy::handler(uri, headers, Query(params), state)
                        .await
                        .into_response()
                },
                |op| op
                    .metrics_tag()
                    .summary("Legacy query endpoint")
                    .description(
                        "**DEPRECATED** - Use `/api/series/{series}/{index}` or `/api/series/bulk` instead.\n\n\
                        Sunset date: 2027-01-01. May be removed earlier in case of abuse.\n\n\
                        Legacy endpoint for querying series. Returns raw data without the SeriesData wrapper."
                    )
                    .deprecated()
                    .json_response::<serde_json::Value>()
                    .not_modified(),
            ),
        )
    }
}
