use brk_types::{Height, Sats};
use vecdb::{
    CachedBoxedVec, CachedReadableVec, CachedVec, ReadableBoxedVec, ReadableCloneableVec,
    ReadableVec, TypedVec,
};

/// Pinned in-memory snapshot of the all-cohort supply.
///
/// Every cohort dominance vec shares this cache. It intentionally bypasses the
/// global cache budget because evicting it would make each lazy read hit disk.
#[derive(Clone)]
pub(crate) struct AllSupplyCache {
    cache: CachedBoxedVec<Height, Sats>,
    source: ReadableBoxedVec<Height, Sats>,
}

impl AllSupplyCache {
    pub(crate) fn new<V>(source: V) -> Self
    where
        V: TypedVec<I = Height, T = Sats>
            + ReadableVec<Height, Sats>
            + Clone
            + Send
            + Sync
            + 'static,
    {
        let cache = CachedVec::wrap(source);
        let source = ReadableCloneableVec::read_only_boxed_clone(&cache);
        let cache = cache.cached_boxed_clone();

        Self { cache, source }
    }

    pub(crate) fn cached_boxed_clone(&self) -> CachedBoxedVec<Height, Sats> {
        self.cache.cached_boxed_clone()
    }

    pub(crate) fn readable_boxed_clone(&self) -> ReadableBoxedVec<Height, Sats> {
        self.source.read_only_boxed_clone()
    }

    pub(crate) fn clear(&self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use brk_types::Version;
    use vecdb::{AnyStoredVec, Database, EagerVec, ImportableVec, PcoVec, ReadOnlyClone, WritableVec};

    use super::*;

    #[test]
    fn clear_refreshes_a_same_length_rewrite() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "brk-all-supply-cache-{}-{suffix}",
            std::process::id()
        ));
        let db = Database::open(&path).unwrap();
        let mut source: EagerVec<PcoVec<Height, Sats>> =
            EagerVec::forced_import(&db, "supply", Version::ONE).unwrap();

        source.push(Sats::new(10));
        source.push(Sats::new(20));
        source.write().unwrap();

        let cache = AllSupplyCache::new(source.read_only_clone());
        let reader = cache.cached_boxed_clone();
        assert_eq!(&*reader.cached(), &[Sats::new(10), Sats::new(20)]);

        source.truncate_if_needed_at(1).unwrap();
        source.push(Sats::new(30));
        source.write().unwrap();

        assert_eq!(&*reader.cached(), &[Sats::new(10), Sats::new(20)]);
        cache.clear();
        assert_eq!(&*reader.cached(), &[Sats::new(10), Sats::new(30)]);

        drop(reader);
        drop(cache);
        drop(source);
        drop(db);
        std::fs::remove_dir_all(path).unwrap();
    }
}
