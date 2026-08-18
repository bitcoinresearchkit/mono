// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use super::{block_index::BlockIndexImpl, meta::ParsedMeta, regions::ParsedRegions};
use crate::{
    Checksum, GlobalTableId, SeqNo,
    cache::Cache,
    file_accessor::FileAccessor,
    table::{IndexBlock, filter::block::FilterBlock},
    tree::inner::TreeId,
};
use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};

pub struct Inner {
    pub path: Arc<PathBuf>,

    pub(crate) tree_id: TreeId,

    #[doc(hidden)]
    pub(crate) file_accessor: FileAccessor,

    /// Parsed metadata
    #[doc(hidden)]
    pub metadata: ParsedMeta,

    /// Parsed region block handles
    #[doc(hidden)]
    pub regions: ParsedRegions,

    /// Translates key (first item of a block) to block offset (address inside file) and (compressed) size
    #[doc(hidden)]
    pub block_index: Arc<BlockIndexImpl>,

    /// Block cache
    ///
    /// Stores index and data blocks
    #[doc(hidden)]
    pub cache: Arc<Cache>,

    /// Pinned filter index (in case of partitioned filters)
    pub(super) pinned_filter_index: Option<IndexBlock>,

    /// Pinned AMQ filter
    pub pinned_filter_block: Option<FilterBlock>,

    /// True when the table was compacted away or dropped
    ///
    /// Open readers keep the table alive until they finish.
    pub is_deleted: AtomicBool,

    pub(super) checksum: Checksum,

    pub(super) global_seqno: SeqNo,
}

impl Inner {
    /// Gets the global table ID.
    #[must_use]
    pub(super) fn global_id(&self) -> GlobalTableId {
        (self.tree_id, self.metadata.id).into()
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        let global_id = self.global_id();

        if self.is_deleted.load(std::sync::atomic::Ordering::Acquire) {
            log::trace!("Cleanup deleted table {global_id:?} at {:?}", self.path);

            if let Err(e) = std::fs::remove_file(&*self.path) {
                log::warn!(
                    "Failed to cleanup deleted table {global_id:?} at {:?}: {e:?}",
                    self.path,
                );
            }

            self.file_accessor.as_descriptor_table().inspect(|d| {
                d.remove_for_table(global_id);
            });
        }
    }
}
