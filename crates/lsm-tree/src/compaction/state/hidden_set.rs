// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

/// The hidden set keeps track of which tables are currently being compacted
///
/// When a table is hidden (being compacted), no other compaction task can include that
/// table, or it will be declined to be run.
///
/// If a compaction task fails, the tables are shown again (removed from the hidden set).
#[derive(Clone, Default)]
pub struct HiddenSet {
    set: rustc_hash::FxHashSet<u32>,
}

impl HiddenSet {
    pub fn hide<T: IntoIterator<Item = u32>>(&mut self, keys: T) {
        self.set.extend(keys);
    }

    pub fn show<T: IntoIterator<Item = u32>>(&mut self, keys: T) {
        for key in keys {
            self.set.remove(&key);
        }
    }

    pub fn is_blocked<T: IntoIterator<Item = u32>>(&self, ids: T) -> bool {
        ids.into_iter().any(|id| self.is_hidden(id))
    }

    pub fn is_hidden(&self, key: u32) -> bool {
        self.set.contains(&key)
    }

    pub fn should_decline_compaction<T: IntoIterator<Item = u32>>(&self, ids: T) -> bool {
        self.is_blocked(ids)
    }
}
