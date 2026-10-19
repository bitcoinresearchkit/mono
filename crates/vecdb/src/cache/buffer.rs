use std::sync::Arc;

use super::{Account, Charge};

#[derive(Debug)]
pub(super) struct Buffer<T> {
    pub(super) values: Vec<T>,
    // Field order releases the values before their budget reservation.
    charge: Option<Charge>,
}

impl<T> Buffer<T> {
    pub(super) fn new(values: Vec<T>) -> Self {
        Self {
            values,
            charge: None,
        }
    }

    /// An allocation's capacity never exceeds `isize::MAX` bytes: no overflow.
    pub(super) fn bytes(&self) -> usize {
        self.values.capacity() * size_of::<T>() + size_of::<Self>() + 2 * size_of::<usize>()
    }

    pub(super) fn admit(&mut self, account: &Arc<Account>) -> bool {
        if self.charge.is_none() {
            self.charge = account.reserve(self.bytes());
        }
        self.charge.is_some()
    }

    pub(super) fn is_charged(&self) -> bool {
        self.charge.is_some()
    }

    pub(super) fn reserve(&mut self, len: usize, limit: usize) -> bool {
        if len <= self.values.capacity() {
            return true;
        }
        let capacity = len
            .checked_next_power_of_two()
            .unwrap_or(len)
            .min(limit.max(len));
        let Some(extra) = (capacity - self.values.capacity()).checked_mul(size_of::<T>()) else {
            return false;
        };
        if let Some(charge) = &mut self.charge
            && !charge.grow(extra)
        {
            return false;
        }
        self.values.reserve_exact(capacity - self.values.len());
        debug_assert_eq!(self.values.capacity(), capacity);
        true
    }
}
