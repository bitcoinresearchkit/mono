//! Source-owned range retention. Cache buffers never escape a read operation.

mod account;
mod budget;
mod budgeted;
mod buffer;
mod charge;
mod directory;
mod none;
mod plan;
mod policy;
mod request;
mod source;
mod table;
mod value;

pub use budget::CacheBudget;
pub use budgeted::Budgeted;
pub use none::NoCache;
pub use policy::CachePolicy;

use account::Account;
use buffer::Buffer;
use charge::Charge;
pub(crate) use request::Request;
use table::Table;
use value::Value;

use std::{
    mem,
    ops::Range,
    result::Result as FoldResult,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering::Relaxed},
    },
};

use parking_lot::{Mutex, RwLock};

use crate::{READ_CHUNK_SIZE, Result, VecValue};
use budget::Reclaim;

/// One cache shared by a source and all of its read-only views.
#[derive(Debug)]
pub struct Cache<T: VecValue> {
    gate: RwLock<()>,
    fill: Mutex<()>,
    table: RwLock<Table<T>>,
    account: Arc<Account>,
    healthy: AtomicBool,
    recently_used: AtomicBool,
}

impl<T: VecValue> Cache<T> {
    fn new(budget: &'static CacheBudget) -> Arc<Self> {
        let cache = Arc::new(Self {
            gate: RwLock::new(()),
            fill: Mutex::new(()),
            table: RwLock::new(Table::new()),
            account: Account::new(budget),
            healthy: AtomicBool::new(true),
            recently_used: AtomicBool::new(false),
        });
        let reclaim: Arc<dyn Reclaim> = cache.clone();
        budget.register(Arc::downgrade(&reclaim));
        cache
    }

    /// Accounted bytes, including buffers still borrowed after eviction.
    fn used(&self) -> usize {
        self.account.used()
    }

    pub(crate) fn try_read_range(
        &self,
        from: usize,
        to: usize,
        len: impl FnOnce() -> usize,
        out: &mut Vec<T>,
    ) -> bool {
        let Some(_gate) = self.gate.try_read_recursive() else {
            return false;
        };
        self.recently_used.store(true, Relaxed);
        self.healthy.load(Relaxed) && from <= to && to <= len() && self.copy_range(from, to, out)
    }

    fn read_scope<R>(&self, read: impl FnOnce() -> R) -> R {
        let _gate = self.gate.read_recursive();
        assert!(self.healthy.load(Relaxed), "unpublished source");
        self.recently_used.store(true, Relaxed);
        read()
    }

    /// A failed source write stays closed until a successful repair/update.
    fn update<R>(&self, from: usize, write: impl FnOnce() -> Result<R>) -> Result<R> {
        let _gate = self.gate.write();
        self.healthy.store(false, Relaxed);
        self.invalidate_from(from);
        let result = write();
        if result.is_ok() {
            self.healthy.store(true, Relaxed);
        }
        result
    }

    /// The loader returns sorted, non-overlapping source-selected ranges that
    /// cover every miss. Physical page size is deliberately not a cache concern.
    fn read(
        &self,
        request: Request<'_>,
        out: &mut Vec<T>,
        load: impl FnMut(&[Range<usize>]) -> Vec<(usize, Vec<T>)>,
    ) {
        self.read_request(request, out, load, self.admissible(request));
    }

    fn get(&self, index: usize, load: impl FnMut(&[Range<usize>]) -> Vec<(usize, Vec<T>)>) -> T {
        {
            let table = self.table.read();
            if let Some((start, values)) = table.at(index)
                && let Some(value) = values.get(index - start)
            {
                return value;
            }
        }
        let mut out = Vec::with_capacity(1);
        self.read(Request::Sorted(&[index]), &mut out, load);
        out.pop()
            .expect("source must cover a valid requested index")
    }

    fn admissible(&self, request: Request<'_>) -> bool {
        match request {
            Request::Range(from, to) => to
                .saturating_sub(from)
                .checked_mul(size_of::<T>())
                .and_then(|bytes| bytes.checked_add(Table::<T>::charge_for(1)))
                .is_some_and(|bytes| bytes <= self.account.budget.limit()),
            _ => true,
        }
    }

    fn read_request(
        &self,
        request: Request<'_>,
        out: &mut Vec<T>,
        mut load: impl FnMut(&[Range<usize>]) -> Vec<(usize, Vec<T>)>,
        retain: bool,
    ) {
        let Some(request) = self.copy_prefix(request, out) else {
            return;
        };
        let _fill = self.fill.lock();
        let cached = self.borrowed(request);
        let missing = plan::missing(request, &cached);
        if missing.is_empty() {
            plan::copy(request, &cached, &[], out);
            return;
        }
        let loaded: Vec<_> = load(&missing)
            .into_iter()
            .filter(|(_, values)| !values.is_empty())
            .map(|(start, values)| (start, Value::from_vec(values)))
            .collect();
        plan::copy(request, &cached, &loaded, out);
        if !retain {
            return;
        }

        // Keep exactly the missing ranges, not decoder overread. Whole matching
        // buffers move into the cache without another allocation or copy.
        let exact = loaded.len() == missing.len()
            && loaded.iter().zip(&missing).all(|((start, values), range)| {
                *start == range.start && values.len() == range.len()
            });
        let offers = if exact {
            loaded
        } else {
            missing
                .into_iter()
                .map(|range| {
                    let value = if range.len() == 1 {
                        let (start, values) = plan::containing(&loaded, range.start)
                            .expect("source loader must cover every cache miss");
                        Value::One(values.slice()[range.start - start].clone())
                    } else {
                        let mut values = Vec::with_capacity(range.len());
                        plan::copy(
                            Request::Range(range.start, range.end),
                            &[],
                            &loaded,
                            &mut values,
                        );
                        Value::from_vec(values)
                    };
                    (range.start, value)
                })
                .collect()
        };
        // Drop temporary reader handles before merging, so an unleased cached
        // buffer can extend in place without copy-on-write.
        drop(cached);
        self.insert(offers);
    }

    fn try_for_each_chunk<E>(
        &self,
        from: usize,
        to: usize,
        mut load: impl FnMut(&[Range<usize>]) -> Vec<(usize, Vec<T>)>,
        mut each: impl FnMut(usize, &[T]) -> FoldResult<(), E>,
    ) -> FoldResult<(), E> {
        if from >= to {
            return Ok(());
        }
        let cached = self.borrowed(Request::Range(from, to));
        let retain = self.admissible(Request::Range(from, to));
        let mut at = from;
        let mut scratch = Vec::new();
        for (start, values) in &cached {
            let gap_end = (*start).min(to);
            while at < gap_end {
                let end = at.saturating_add(READ_CHUNK_SIZE).min(gap_end);
                scratch.clear();
                self.read_request(Request::Range(at, end), &mut scratch, &mut load, retain);
                each(at, &scratch)?;
                at = end;
            }
            let end = (*start + values.len()).min(to);
            if at < end {
                each(at, &values.slice()[at - *start..end - *start])?;
                at = end;
            }
        }
        while at < to {
            let end = at.saturating_add(READ_CHUNK_SIZE).min(to);
            scratch.clear();
            self.read_request(Request::Range(at, end), &mut scratch, &mut load, retain);
            each(at, &scratch)?;
            at = end;
        }
        Ok(())
    }
}

impl<T: VecValue> Reclaim for Cache<T> {
    fn try_clear(&self) {
        if self.used() == 0 || self.recently_used.swap(false, Relaxed) {
            return;
        }
        let old = self
            .table
            .try_write()
            .map(|mut table| mem::replace(&mut *table, Table::new()));
        drop(old);
    }

    fn clear(&self) {
        let old = mem::replace(&mut *self.table.write(), Table::new());
        drop(old);
    }
}
