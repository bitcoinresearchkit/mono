// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use crate::SequenceNumberCounter;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GlobalTableId(u32, u32);

impl From<(u32, u32)> for GlobalTableId {
    fn from((tree_id, table_id): (u32, u32)) -> Self {
        Self(tree_id, table_id)
    }
}

#[expect(
    clippy::expect_used,
    reason = "exhausting the complete u32 table ID space is unrecoverable"
)]
pub fn next_table_id(counter: &SequenceNumberCounter) -> u32 {
    counter.next().try_into().expect("ran out of table IDs")
}
