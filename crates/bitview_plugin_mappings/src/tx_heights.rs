use std::sync::Arc;

use bitview_plugin_indexer::Indexer;
use brk_types::{Height, RangeMap, TxIndex};
use parking_lot::RwLock;
use vecdb::{AnyVec, ReadableVec, VecIndex};

/// Reverse mapping from `TxIndex` → `Height` via binary search on block boundaries.
///
/// Built from `first_tx_index` (the first TxIndex in each block). A floor lookup
/// on any TxIndex gives the block height that contains it.
///
/// Wrapped in `Arc<RwLock<>>` so the compute thread can extend it while
/// query threads read concurrently — the inner `RangeMap` is purely in-memory
/// and wouldn't stay current through mmap like PcoVec/BytesVec do.
#[derive(Clone)]
pub struct TxHeights(Arc<RwLock<RangeMap<TxIndex, Height>>>);

impl TxHeights {
    /// Build from the full `first_tx_index` vec at startup.
    pub fn init(indexer: &Indexer) -> Self {
        let len = indexer.vecs().transactions.first_tx_index.len();
        let entries: Vec<TxIndex> = if len > 0 {
            indexer
                .vecs()
                .transactions
                .first_tx_index
                .collect_range_at(0, len)
        } else {
            Vec::new()
        };
        Self(Arc::new(RwLock::new(RangeMap::from(entries))))
    }

    /// Extend with new blocks since last call. Truncates on reorg.
    pub fn update(&self, indexer: &Indexer, reorg_height: Height) {
        let mut inner = self.0.write();
        let reorg_len = reorg_height.to_usize();
        if inner.len() > reorg_len {
            inner.truncate(reorg_len);
        }
        let target_len = indexer.vecs().transactions.first_tx_index.len();
        let current_len = inner.len();
        if current_len < target_len {
            let new_entries: Vec<TxIndex> = indexer
                .vecs()
                .transactions
                .first_tx_index
                .collect_range_at(current_len, target_len);
            for entry in new_entries {
                inner.push(entry);
            }
        }
    }

    /// Look up the block height for a given tx_index.
    #[inline]
    pub fn get_shared(&self, tx_index: TxIndex) -> Option<Height> {
        self.0.read().get_shared(tx_index)
    }
}
