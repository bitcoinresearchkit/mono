use std::{sync::Arc, thread::available_parallelism};

use bitcoin::hashes::{Hash, HashEngine, sha256};
use bitview_query::Query;
use bitview_types::{Limit, Pagination, SeriesInfo};
use serde_json::to_writer;
use tokio::sync::Semaphore;

use crate::{CacheParams, prepared_json::PreparedJson};

pub struct SeriesBodies {
    pub catalog: PreparedJson,
    pub count: PreparedJson,
    pub indexes: PreparedJson,
    pub list: CacheParams,
    pub search: CacheParams,
    pub info: CacheParams,
    pub search_query: Arc<Semaphore>,
    pub data_query: Arc<Semaphore>,
}

impl SeriesBodies {
    pub fn new(query: &Query) -> Self {
        let query_capacity = available_parallelism().map_or(1, |count| count.get());
        let catalog = PreparedJson::new(query.series_catalog());
        let search = search_params(catalog.bytes(), query.vecs().series_names());
        Self {
            catalog,
            count: PreparedJson::new(query.series_count()),
            indexes: PreparedJson::new(query.indexes()),
            list: list_params(query.vecs().series_names()),
            search,
            info: info_params(query.vecs().series_names().iter().map(|&name| {
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
fn info_params<'a>(entries: impl IntoIterator<Item = (&'a str, SeriesInfo)>) -> CacheParams {
    let mut engine = sha256::HashEngine::default();
    let mut bytes = Vec::new();
    for (name, info) in entries {
        hash_bytes(&mut engine, name.as_bytes());
        bytes.clear();
        to_writer(&mut bytes, &info).expect("serializable series info");
        hash_bytes(&mut engine, &bytes);
    }
    CacheParams::revalidate(format!("info1-{}", sha256::Hash::from_engine(engine)).into())
}

/// URI-scoped revision, not an assertion that different pages have equal bodies.
/// Bump list1 when the response schema or pagination derivation changes.
fn list_params(names: &[&str]) -> CacheParams {
    let mut engine = sha256::Hash::engine();
    for name in names {
        hash_bytes(&mut engine, name.as_bytes());
    }
    CacheParams::revalidate(
        format!(
            "list1-{}-{}-{}",
            Pagination::DEFAULT_PER_PAGE,
            Pagination::MAX_PER_PAGE,
            sha256::Hash::from_engine(engine),
        )
        .into(),
    )
}

/// URI-scoped search revision. Bump search2 for changes to matching, normalization,
/// ranking, or output formatting, including changes to QuickMatch defaults.
fn search_params(catalog: &[u8], names: &[&str]) -> CacheParams {
    let mut engine = sha256::Hash::engine();
    hash_bytes(&mut engine, catalog);
    engine.input(&(names.len() as u64).to_le_bytes());
    for name in names {
        hash_bytes(&mut engine, name.as_bytes());
    }
    CacheParams::revalidate(
        format!(
            "search2-{}-{}",
            Limit::DEFAULT,
            sha256::Hash::from_engine(engine)
        )
        .into(),
    )
}

fn hash_bytes(engine: &mut sha256::HashEngine, bytes: &[u8]) {
    engine.input(&(bytes.len() as u64).to_le_bytes());
    engine.input(bytes);
}
