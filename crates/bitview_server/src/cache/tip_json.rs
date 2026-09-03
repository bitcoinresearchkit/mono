use std::{future::Future, hash::Hash, sync::Arc};

use axum::body::Bytes;
use brk_error::Result;
use brk_types::BlockHashPrefix;
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use tokio::sync::Mutex;

struct TipEntries<K> {
    tip: Option<BlockHashPrefix>,
    values: FxHashMap<K, Bytes>,
    builds: FxHashMap<K, Arc<Mutex<()>>>,
}

impl<K> Default for TipEntries<K> {
    fn default() -> Self {
        Self {
            tip: None,
            values: FxHashMap::default(),
            builds: FxHashMap::default(),
        }
    }
}

/// Serialized JSON representations valid for one exact chain tip.
///
/// Hits never enter the blocking query pool. Concurrent misses for one key
/// share a build, while different keys can build in parallel.
#[derive(Clone)]
pub(crate) struct TipJsonCache<K>(Arc<RwLock<TipEntries<K>>>);

impl<K> Default for TipJsonCache<K> {
    fn default() -> Self {
        Self(Arc::new(RwLock::default()))
    }
}

impl<K> TipJsonCache<K>
where
    K: Clone + Eq + Hash,
{
    pub(crate) fn current(&self, key: &K, tip: BlockHashPrefix) -> Option<Bytes> {
        let entries = self.0.read();
        if entries.tip != Some(tip) {
            return None;
        }
        entries.values.get(key).cloned()
    }

    fn build_lock(&self, key: &K, tip: BlockHashPrefix) -> Arc<Mutex<()>> {
        let mut entries = self.0.write();
        if entries.tip != Some(tip) {
            entries.tip = Some(tip);
            entries.values.clear();
            entries.builds.clear();
        }
        Arc::clone(
            entries
                .builds
                .entry(key.clone())
                .or_insert_with(|| Arc::new(Mutex::new(()))),
        )
    }

    pub(crate) async fn get_or_try_insert_with<F, Fut>(
        &self,
        key: K,
        tip: BlockHashPrefix,
        build: F,
    ) -> Result<Bytes>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Bytes>>,
    {
        if let Some(bytes) = self.current(&key, tip) {
            return Ok(bytes);
        }

        let build_lock = self.build_lock(&key, tip);
        let _build = build_lock.lock().await;
        if let Some(bytes) = self.current(&key, tip) {
            return Ok(bytes);
        }

        let bytes = build().await?;
        let mut entries = self.0.write();
        if entries.tip == Some(tip) {
            entries.values.insert(key, bytes.clone());
        }
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use brk_error::Error;
    use tokio::sync::oneshot;

    use super::*;

    fn tip(value: u64) -> BlockHashPrefix {
        BlockHashPrefix::from(value)
    }

    async fn tracked_build(
        active: Arc<AtomicUsize>,
        maximum: Arc<AtomicUsize>,
        bytes: &'static [u8],
    ) -> Result<Bytes> {
        let current = active.fetch_add(1, Ordering::Relaxed) + 1;
        maximum.fetch_max(current, Ordering::Relaxed);
        tokio::task::yield_now().await;
        active.fetch_sub(1, Ordering::Relaxed);
        Ok(Bytes::from_static(bytes))
    }

    #[tokio::test]
    async fn builds_different_keys_concurrently() {
        let cache = TipJsonCache::default();
        let active = Arc::new(AtomicUsize::new(0));
        let maximum = Arc::new(AtomicUsize::new(0));

        let first_cache = cache.clone();
        let first_active = Arc::clone(&active);
        let first_maximum = Arc::clone(&maximum);
        let first = async move {
            first_cache
                .get_or_try_insert_with(1, tip(1), move || {
                    tracked_build(first_active, first_maximum, b"one")
                })
                .await
        };

        let second_cache = cache.clone();
        let second_active = Arc::clone(&active);
        let second_maximum = Arc::clone(&maximum);
        let second = async move {
            second_cache
                .get_or_try_insert_with(2, tip(1), move || {
                    tracked_build(second_active, second_maximum, b"two")
                })
                .await
        };

        let (first, second) = tokio::join!(first, second);
        assert_eq!(first.unwrap(), Bytes::from_static(b"one"));
        assert_eq!(second.unwrap(), Bytes::from_static(b"two"));
        assert_eq!(maximum.load(Ordering::Relaxed), 2);
    }

    #[tokio::test]
    async fn older_build_cannot_replace_a_new_tip() {
        let cache = TipJsonCache::default();
        let (started_tx, started_rx) = oneshot::channel();
        let (release_tx, release_rx) = oneshot::channel();

        let older_cache = cache.clone();
        let older = async move {
            older_cache
                .get_or_try_insert_with(1, tip(1), move || async move {
                    started_tx.send(()).unwrap();
                    release_rx.await.unwrap();
                    Ok(Bytes::from_static(b"older"))
                })
                .await
        };

        let newer_cache = cache.clone();
        let newer = async move {
            started_rx.await.unwrap();
            let bytes = newer_cache
                .get_or_try_insert_with(1, tip(2), || async { Ok(Bytes::from_static(b"newer")) })
                .await;
            release_tx.send(()).unwrap();
            bytes
        };

        let (older, newer) = tokio::join!(older, newer);
        assert_eq!(older.unwrap(), Bytes::from_static(b"older"));
        assert_eq!(newer.unwrap(), Bytes::from_static(b"newer"));
        assert_eq!(cache.current(&1, tip(2)).unwrap(), b"newer"[..]);
        assert!(cache.current(&1, tip(1)).is_none());
    }

    #[tokio::test]
    async fn caches_once_per_key_and_tip_without_caching_errors() {
        let cache = TipJsonCache::default();
        let builds = Arc::new(AtomicUsize::new(0));

        let first_cache = cache.clone();
        let first_builds = Arc::clone(&builds);
        let first = async move {
            first_cache
                .get_or_try_insert_with(1, tip(1), move || async move {
                    first_builds.fetch_add(1, Ordering::Relaxed);
                    tokio::task::yield_now().await;
                    Ok(Bytes::from_static(b"one"))
                })
                .await
        };

        let second_cache = cache.clone();
        let second_builds = Arc::clone(&builds);
        let second = async move {
            second_cache
                .get_or_try_insert_with(1, tip(1), move || async move {
                    second_builds.fetch_add(1, Ordering::Relaxed);
                    Ok(Bytes::from_static(b"duplicate"))
                })
                .await
        };

        let (first, second) = tokio::join!(first, second);
        assert_eq!(first.unwrap(), Bytes::from_static(b"one"));
        assert_eq!(second.unwrap(), Bytes::from_static(b"one"));
        assert_eq!(builds.load(Ordering::Relaxed), 1);

        let changed_builds = Arc::clone(&builds);
        let changed = cache
            .get_or_try_insert_with(1, tip(2), move || async move {
                changed_builds.fetch_add(1, Ordering::Relaxed);
                Ok(Bytes::from_static(b"two"))
            })
            .await
            .unwrap();
        assert_eq!(changed, Bytes::from_static(b"two"));
        assert_eq!(builds.load(Ordering::Relaxed), 2);

        assert!(
            cache
                .get_or_try_insert_with(2, tip(2), || async {
                    Err(Error::Internal("expected test error"))
                })
                .await
                .is_err()
        );
        let recovered_builds = Arc::clone(&builds);
        let recovered = cache
            .get_or_try_insert_with(2, tip(2), move || async move {
                recovered_builds.fetch_add(1, Ordering::Relaxed);
                Ok(Bytes::from_static(b"recovered"))
            })
            .await
            .unwrap();
        assert_eq!(recovered, Bytes::from_static(b"recovered"));
        assert_eq!(builds.load(Ordering::Relaxed), 3);
    }
}
