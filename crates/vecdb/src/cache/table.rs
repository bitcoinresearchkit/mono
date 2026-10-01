use std::{ops::Deref, sync::Arc};

use super::{Account, Charge, Value};

/// Sorted, non-overlapping retained spans. Single values live inline.
#[derive(Debug)]
pub(super) struct Table<T> {
    pub(super) entries: Vec<(usize, Value<T>)>,
    charge: Option<Charge>,
}

impl<T: Clone> Table<T> {
    pub(super) fn new() -> Self {
        Self {
            entries: Vec::new(),
            charge: None,
        }
    }

    pub(super) fn charge_for(capacity: usize) -> usize {
        capacity
            .checked_mul(size_of::<(usize, Value<T>)>())
            .expect("cache directory overflow")
    }

    pub(super) fn first(&self, from: usize) -> usize {
        self.entries
            .partition_point(|(at, value)| *at + value.len() <= from)
    }

    pub(super) fn at(&self, index: usize) -> Option<(usize, &Value<T>)> {
        let at = self
            .entries
            .partition_point(|(at, _)| *at <= index)
            .checked_sub(1)?;
        let (start, value) = &self.entries[at];
        (index - start < value.len()).then_some((*start, value))
    }

    fn reserve(&mut self, len: usize, account: &Arc<Account>) -> bool {
        if len <= self.entries.capacity() {
            return true;
        }
        let capacity = len.checked_next_power_of_two().unwrap_or(len);
        let bytes = Self::charge_for(capacity - self.entries.capacity());
        let Some(charge) = account.reserve(bytes) else {
            return false;
        };
        self.entries.reserve_exact(capacity - self.entries.len());
        debug_assert_eq!(self.entries.capacity(), capacity);
        if let Some(current) = &mut self.charge {
            current.merge(charge);
        } else {
            self.charge = Some(charge);
        }
        true
    }

    /// Merge backward into spare capacity, moving each entry at most once.
    pub(super) fn insert(&mut self, mut offers: Vec<(usize, Value<T>)>, account: &Arc<Account>) {
        if offers.is_empty() {
            return;
        }
        let at = self
            .entries
            .partition_point(|(start, _)| *start < offers[0].0);
        let len = self.entries.len() + offers.len();
        if !self.reserve(len, account) {
            return;
        }
        if at == self.entries.len() || offers.len() == 1 {
            self.entries.splice(at..at, offers);
        } else {
            let mut end = len;
            while let Some((start, _)) = offers.last() {
                let value = if self.entries.last().is_some_and(|(at, _)| at > start) {
                    self.entries.pop().unwrap()
                } else {
                    offers.pop().unwrap()
                };
                end -= 1;
                let offset = end - self.entries.len();
                self.entries.spare_capacity_mut()[offset].write(value);
            }
            // SAFETY: reserve(len) provided the capacity. Each pop transfers one
            // owned entry into the next uninitialized suffix slot; no allocation,
            // cloning, or user code runs during the merge. With offers exhausted,
            // the untouched prefix and initialized suffix cover exactly 0..len.
            unsafe { self.entries.set_len(len) };
        }
    }
}

impl<T> Deref for Table<T> {
    type Target = [(usize, Value<T>)];
    fn deref(&self) -> &Self::Target {
        &self.entries
    }
}
