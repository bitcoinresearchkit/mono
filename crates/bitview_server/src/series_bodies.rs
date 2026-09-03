use std::sync::Arc;

use axum::body::Bytes;
use bitview_query::AsyncQuery;
use serde::Serialize;

#[derive(Clone)]
pub(crate) struct SeriesBodies(Arc<SeriesBodyBytes>);

struct SeriesBodyBytes {
    catalog: Bytes,
    count: Bytes,
    indexes: Bytes,
}

impl SeriesBodies {
    pub fn new(query: &AsyncQuery) -> Self {
        Self(Arc::new(query.sync(|query| SeriesBodyBytes {
            catalog: serialize(query.series_catalog()),
            count: serialize(query.series_count()),
            indexes: serialize(query.indexes()),
        })))
    }

    pub fn catalog(&self) -> &Bytes {
        &self.0.catalog
    }

    pub fn count(&self) -> &Bytes {
        &self.0.count
    }

    pub fn indexes(&self) -> &Bytes {
        &self.0.indexes
    }
}

fn serialize(value: impl Serialize) -> Bytes {
    Bytes::from(serde_json::to_vec(&value).unwrap())
}
