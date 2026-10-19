// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::{
    fs,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use log::{trace, warn};

use super::{block_index::BlockIndexImpl, meta::ParsedMeta, regions::ParsedRegions};
use crate::{
    GlobalTableId,
    table::{IndexBlock, filter::standard_bloom::StandardBloomFilterReader, util::BlockSource},
};

pub struct Inner {
    /// The table file, its descriptor and block caches
    #[doc(hidden)]
    pub source: BlockSource,

    /// Parsed metadata
    #[doc(hidden)]
    pub metadata: ParsedMeta,

    /// Parsed region block handles
    #[doc(hidden)]
    pub regions: ParsedRegions,

    /// Translates key (first item of a block) to block offset (address inside file) and (compressed) size
    #[doc(hidden)]
    pub block_index: Arc<BlockIndexImpl>,

    /// Pinned filter index (in case of partitioned filters)
    pub pinned_filter_index: Option<IndexBlock>,

    /// Pinned AMQ filter
    pub pinned_filter_block: Option<StandardBloomFilterReader>,

    /// True when the table was compacted away or dropped
    ///
    /// Open readers keep the table alive until they finish.
    pub is_deleted: AtomicBool,

    pub global_seqno: u64,
}

impl Inner {
    /// Gets the global table ID.
    #[must_use]
    pub fn global_id(&self) -> GlobalTableId {
        self.source.table_id
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        let global_id = self.global_id();

        if self.is_deleted.load(Ordering::Acquire) {
            let path = &self.source.path;
            trace!("Cleanup deleted table {global_id:?} at {path:?}");

            if let Err(e) = fs::remove_file(&**path) {
                warn!("Failed to cleanup deleted table {global_id:?} at {path:?}: {e:?}");
            }

            self.source.descriptor_table.remove_for_table(global_id);
        }
    }
}
