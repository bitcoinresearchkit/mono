use std::{
    hash::{Hash, Hasher},
    sync::Arc,
    thread::available_parallelism,
};

use bitview_query::Query;
use bitview_types::{PageSize, Pagination, SearchQuery, SeriesInfo};
use brk_types::TagHasher;
use serde_json::to_vec;
use tokio::sync::Semaphore;

use crate::{etag::Etag, prepared_json::PreparedJson};

pub struct SeriesBodies {
    pub catalog: PreparedJson,
    pub count: PreparedJson,
    pub indexes: PreparedJson,
    pub list: Etag,
    pub search: Etag,
    pub info: Etag,
    pub search_query: Arc<Semaphore>,
    pub data_query: Arc<Semaphore>,
}

impl SeriesBodies {
    pub fn new(query: &Query) -> Self {
        let query_capacity = available_parallelism().map_or(1, |count| count.get());
        let catalog = PreparedJson::new(query.series_catalog());
        let search = search_tag(catalog.bytes(), query.registry().series_names());
        Self {
            catalog,
            count: PreparedJson::new(query.series_count()),
            indexes: PreparedJson::new(query.indexes()),
            list: list_tag(query.registry().series_names()),
            search,
            info: info_tag(query.registry().series_names().iter().map(|&name| {
                (
                    name,
                    query
                        .series_info(&name.into())
                        .expect("registered series info"),
                )
            })),
            search_query: Arc::new(Semaphore::new(query_capacity)),
            data_query: Arc::new(Semaphore::new(query_capacity)),
        }
    }
}

/// URI-scoped metadata revision. Bump info1 if name resolution semantics change.
fn info_tag<'a>(entries: impl IntoIterator<Item = (&'a str, SeriesInfo)>) -> Etag {
    let mut hasher = TagHasher::default();
    for (name, info) in entries {
        name.hash(&mut hasher);
        to_vec(&info).expect("serializable series info").hash(&mut hasher);
    }
    format!("info1-{:x}", hasher.finish()).into()
}

/// URI-scoped revision, not an assertion that different pages have equal bodies.
/// Bump list1 when the response schema or pagination derivation changes.
fn list_tag(names: &[&str]) -> Etag {
    let mut hasher = TagHasher::default();
    names.hash(&mut hasher);
    format!(
        "list1-{}-{}-{:x}",
        *Pagination::DEFAULT_PER_PAGE,
        PageSize::MAX,
        hasher.finish(),
    )
    .into()
}

/// URI-scoped search revision. Bump search2 for changes to matching, normalization,
/// ranking, or output formatting, including changes to QuickMatch defaults.
fn search_tag(catalog: &[u8], names: &[&str]) -> Etag {
    let mut hasher = TagHasher::default();
    catalog.hash(&mut hasher);
    names.hash(&mut hasher);
    format!(
        "search2-{}-{:x}",
        *SearchQuery::DEFAULT_LIMIT,
        hasher.finish()
    )
    .into()
}
