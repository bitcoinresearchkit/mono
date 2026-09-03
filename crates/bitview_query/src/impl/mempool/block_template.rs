use std::sync::Arc;

use brk_mempool::{BlockTemplateSource, Mempool, ResolvedBlockTemplateDiff};
use brk_types::NextBlockHash;
use parking_lot::RwLock;
use rustc_hash::FxHashMap;

use crate::{RepresentationId, r#impl::CachedJson};

struct CachedBlockTemplate {
    source: BlockTemplateSource,
    json: CachedJson,
}

struct CachedBlockTemplateDiffs {
    source: BlockTemplateSource,
    entries: FxHashMap<NextBlockHash, CachedJson>,
}

pub enum BlockTemplateDiffPreflight {
    Cached(Arc<[u8]>, RepresentationId),
    Resolved(ResolvedBlockTemplateDiff),
}

#[derive(Default)]
pub(crate) struct BlockTemplateCache {
    entry: RwLock<Option<CachedBlockTemplate>>,
    diffs: RwLock<Option<CachedBlockTemplateDiffs>>,
}

impl BlockTemplateCache {
    pub(super) fn current(&self, mempool: &Mempool) -> Option<(Arc<[u8]>, RepresentationId)> {
        let entry = self.entry.try_read()?;
        let entry = entry.as_ref()?;
        let source = mempool.block_template_source();
        (entry.source == source).then(|| entry.json.value())
    }

    pub(super) fn get_or_build(&self, mempool: &Mempool) -> (Arc<[u8]>, RepresentationId) {
        let mut cached = self.entry.write();
        if let Some(entry) = cached.as_ref() {
            let source = mempool.block_template_source();
            if entry.source == source {
                return entry.json.value();
            }
        }

        let (template, source) = mempool.block_template_with_source();
        let json = CachedJson::serialize(&template);
        let value = json.value();
        *cached = Some(CachedBlockTemplate { source, json });
        value
    }

    pub(super) fn current_diff(
        &self,
        mempool: &Mempool,
        since: NextBlockHash,
    ) -> Option<(Arc<[u8]>, RepresentationId)> {
        let cached = self.diffs.try_read()?;
        let cached = cached.as_ref()?;
        let entry = cached.entries.get(&since)?;
        let source = mempool.block_template_source();
        (cached.source == source).then(|| entry.value())
    }

    pub(super) fn get_or_build_diff(
        &self,
        mempool: &Mempool,
        resolved: ResolvedBlockTemplateDiff,
    ) -> (Arc<[u8]>, RepresentationId) {
        let since = resolved.since();
        let expected_source = resolved.source().clone();
        let mut cached = self.diffs.write();
        if let Some(cached) = cached.as_ref() {
            let source = mempool.block_template_source();
            if cached.source == source
                && let Some(entry) = cached.entries.get(&since)
            {
                return entry.value();
            }
        }

        let (diff, source) = mempool.block_template_diff_resolved(resolved);
        let json = CachedJson::serialize(&diff);
        let value = json.value();
        if source == expected_source {
            if cached.as_ref().is_none_or(|cached| cached.source != source) {
                *cached = Some(CachedBlockTemplateDiffs {
                    source: source.clone(),
                    entries: FxHashMap::default(),
                });
            }
            cached.as_mut().unwrap().entries.insert(since, json);
        }
        value
    }
}

#[cfg(test)]
mod tests {
    use brk_rpc::{Auth, Client};

    use super::*;
    use crate::representation_id::content_hash;

    fn identity_hash(identity: RepresentationId) -> u64 {
        match identity {
            RepresentationId::Content(hash) => hash,
            RepresentationId::Block { .. } => panic!("block template must be content-identified"),
        }
    }

    #[test]
    fn caches_one_exact_representation_per_source() {
        let client = Client::new("http://127.0.0.1:1", Auth::None).unwrap();
        let mempool = Mempool::new(&client);
        let cache = BlockTemplateCache::default();
        assert!(cache.current(&mempool).is_none());

        let (first_bytes, first_identity) = cache.get_or_build(&mempool);
        let (cached_bytes, cached_identity) = cache.current(&mempool).expect("cached template");
        let (second_bytes, second_identity) = cache.get_or_build(&mempool);

        assert!(Arc::ptr_eq(&first_bytes, &cached_bytes));
        assert!(Arc::ptr_eq(&first_bytes, &second_bytes));
        assert_eq!(identity_hash(first_identity), content_hash(&first_bytes));
        assert_eq!(
            identity_hash(cached_identity),
            identity_hash(first_identity)
        );
        assert_eq!(
            identity_hash(second_identity),
            identity_hash(first_identity)
        );
    }
}
