use std::{slice, sync::Arc};

use super::{Account, Buffer};

/// No allocation or reference count for isolated cached values.
#[derive(Clone, Debug)]
pub(super) enum Value<T> {
    One(T),
    Many(Arc<Buffer<T>>, usize),
}

impl<T: Clone> Value<T> {
    pub(super) fn from_vec(mut values: Vec<T>) -> Self {
        debug_assert!(!values.is_empty());
        if values.len() == 1 {
            Self::One(values.pop().unwrap())
        } else {
            let len = values.len();
            Self::Many(Arc::new(Buffer::new(values)), len)
        }
    }

    pub(super) fn slice(&self) -> &[T] {
        match self {
            Self::One(value) => slice::from_ref(value),
            Self::Many(buffer, len) => &buffer.values[..*len],
        }
    }

    pub(super) fn len(&self) -> usize {
        match self {
            Self::One(_) => 1,
            Self::Many(_, len) => *len,
        }
    }

    pub(super) fn get(&self, index: usize) -> Option<T> {
        self.slice().get(index).cloned()
    }

    pub(super) fn allocation_bytes(&self) -> usize {
        match self {
            Self::One(_) => 0,
            Self::Many(buffer, _) => buffer.bytes(),
        }
    }

    pub(super) fn admit(&mut self, account: &Arc<Account>) -> bool {
        match self {
            Self::One(_) => true,
            Self::Many(buffer, _) if buffer.is_charged() => true,
            Self::Many(buffer, _) => {
                Arc::get_mut(buffer).is_some_and(|buffer| buffer.admit(account))
            }
        }
    }

    /// Extend only a uniquely owned tail; never copy an existing range to join neighbors.
    pub(super) fn try_append(
        &mut self,
        values: &[T],
        limit: usize,
        account: &Arc<Account>,
    ) -> bool {
        let Some(len) = self
            .len()
            .checked_add(values.len())
            .filter(|len| *len <= limit)
        else {
            return false;
        };
        match self {
            Self::One(value) => {
                let mut buffer = Buffer::new(Vec::new());
                if !buffer.admit(account) || !buffer.reserve(len, limit) {
                    return false;
                }
                buffer.values.push(value.clone());
                buffer.values.extend_from_slice(values);
                *self = Self::Many(Arc::new(buffer), len);
            }
            Self::Many(buffer, valid) => {
                let Some(buffer) = Arc::get_mut(buffer) else {
                    return false;
                };
                buffer.values.truncate(*valid);
                if !buffer.reserve(len, limit) {
                    return false;
                }
                buffer.values.extend_from_slice(values);
                *valid = len;
            }
        }
        true
    }

    /// Keep the prefix and its allocation, even if a reader still owns the old view.
    pub(super) fn truncate(&mut self, len: usize) {
        debug_assert!(len > 0 && len <= self.len());
        if let Self::Many(_, valid) = self {
            *valid = len;
        }
    }

    /// Bounded reverse point fills must not leave one span per adjacent value.
    pub(super) fn try_prepend(
        &mut self,
        values: &[T],
        limit: usize,
        account: &Arc<Account>,
    ) -> bool {
        let Some(len) = self
            .len()
            .checked_add(values.len())
            .filter(|len| *len <= limit)
        else {
            return false;
        };
        match self {
            Self::One(value) => {
                let mut prefix = values.to_vec();
                prefix.push(value.clone());
                let mut joined = Self::from_vec(prefix);
                if !joined.admit(account) {
                    return false;
                }
                *self = joined;
            }
            Self::Many(buffer, valid) => {
                let Some(buffer) = Arc::get_mut(buffer) else {
                    return false;
                };
                // Clone before modifying the retained prefix, in case T::clone panics.
                let prefix = values.to_vec();
                if !buffer.reserve(len, limit) {
                    return false;
                }
                buffer.values.truncate(*valid);
                buffer.values.splice(..0, prefix);
                *valid = len;
            }
        }
        true
    }
}
