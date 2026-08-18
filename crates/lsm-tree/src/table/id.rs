// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use crate::{SequenceNumberCounter, tree::inner::TreeId};

pub type TableId = u32;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GlobalTableId(TreeId, TableId);

impl From<(TreeId, TableId)> for GlobalTableId {
    fn from((tree_id, table_id): (TreeId, TableId)) -> Self {
        Self(tree_id, table_id)
    }
}

#[expect(
    clippy::expect_used,
    reason = "exhausting the complete u32 table ID space is unrecoverable"
)]
pub fn next_table_id(counter: &SequenceNumberCounter) -> TableId {
    counter.next().try_into().expect("ran out of table IDs")
}
