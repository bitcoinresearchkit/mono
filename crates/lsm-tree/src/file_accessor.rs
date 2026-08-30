// Copyright (c) 2025-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use crate::GlobalTableId;
use crate::descriptor_table::DescriptorTable;
use std::{fs::File, path::Path, sync::Arc};

/// Allows accessing a table file (either cached or pinned)
#[derive(Clone)]
pub enum FileAccessor {
    /// Pinned file descriptor
    ///
    /// This is used in case file descriptor cache is `None` (to skip cache lookups)
    File(Arc<File>),

    /// Access to file descriptor cache
    DescriptorTable(Arc<DescriptorTable>),
}

impl FileAccessor {
    #[must_use]
    pub fn as_descriptor_table(&self) -> Option<&DescriptorTable> {
        match self {
            Self::DescriptorTable(d) => Some(d),
            Self::File(_) => None,
        }
    }

    pub fn access_or_open(
        &self,
        table_id: GlobalTableId,
        path: &Path,
    ) -> std::io::Result<Arc<File>> {
        match self {
            Self::File(fd) => Ok(fd.clone()),
            Self::DescriptorTable(descriptor_table) => {
                descriptor_table.access_or_open(table_id, path)
            }
        }
    }
}

impl std::fmt::Debug for FileAccessor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::File(_) => write!(f, "FileAccessor::Pinned"),
            Self::DescriptorTable(_) => {
                write!(f, "FileAccessor::Cached")
            }
        }
    }
}
