use std::sync::Arc;

use axum::body::Bytes;
use brk_error::{Error, Result};
use brk_types::{Cohort, UrpdAggregation, UrpdWeight};

use super::TipJsonCache;

const MAX_LATEST_VALUE_BYTES: usize = 64 * 1024 * 1024;

struct UrpdCacheData {
    cohorts: Box<[Cohort]>,
    cohorts_body: Bytes,
    available: Box<str>,
    latest: TipJsonCache<(usize, UrpdAggregation, UrpdWeight)>,
}

#[derive(Clone)]
pub(crate) struct UrpdCaches(Arc<UrpdCacheData>);

impl UrpdCaches {
    pub(crate) fn new(mut cohorts: Vec<Cohort>) -> Result<Self> {
        cohorts.sort_unstable();
        cohorts.dedup();
        let cohorts_body = Bytes::from(serde_json::to_vec(&cohorts)?);
        let available = cohorts
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ")
            .into_boxed_str();

        Ok(Self(Arc::new(UrpdCacheData {
            cohorts: cohorts.into_boxed_slice(),
            cohorts_body,
            available,
            latest: TipJsonCache::with_max_value_bytes(MAX_LATEST_VALUE_BYTES),
        })))
    }

    pub(crate) fn cohorts_body(&self) -> &Bytes {
        &self.0.cohorts_body
    }

    pub(crate) fn cohort_index(&self, cohort: &Cohort) -> Result<usize> {
        if let Ok(index) = self.0.cohorts.binary_search(cohort) {
            return Ok(index);
        }
        Err(Error::NotFound(format!(
            "Unknown cohort '{cohort}'. Available: {}",
            self.0.available
        )))
    }

    pub(crate) fn latest(&self) -> &TipJsonCache<(usize, UrpdAggregation, UrpdWeight)> {
        &self.0.latest
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cohort(name: &str) -> Cohort {
        Cohort::new(name).unwrap()
    }

    #[test]
    fn catalog_is_sorted_deduplicated_and_validates_without_querying() {
        let caches = UrpdCaches::new(vec![cohort("sth"), cohort("all"), cohort("all")]).unwrap();

        assert_eq!(caches.cohorts_body(), b"[\"all\",\"sth\"]".as_slice());
        assert_eq!(caches.cohort_index(&cohort("all")).unwrap(), 0);
        assert_eq!(
            caches
                .cohort_index(&cohort("unknown"))
                .unwrap_err()
                .to_string(),
            "Unknown cohort 'unknown'. Available: all, sth"
        );
    }
}
