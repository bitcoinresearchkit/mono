use std::sync::Arc;

use brk_types::{Cents, Height, StoredU64, Version};
use vecdb::{
    AnyVec, CachedBoxedVec, PrintableIndex, ReadableBoxedVec, ReadableVec, TypedVec, VecIndex,
    short_type_name,
};

mod prefix_sum;

pub use prefix_sum::*;

#[derive(Clone)]
pub struct LazySmaVec {
    name: Arc<str>,
    version: Version,
    window_starts: ReadableBoxedVec<Height, Height>,
    prefix_sum: CachedBoxedVec<Height, StoredU64>,
}

impl LazySmaVec {
    pub fn new(
        name: &str,
        version: Version,
        window_starts: ReadableBoxedVec<Height, Height>,
        prefix_sum: CachedBoxedVec<Height, StoredU64>,
    ) -> Self {
        Self {
            name: Arc::from(name),
            version,
            window_starts,
            prefix_sum,
        }
    }

    fn average(prefix_sum: &[StoredU64], index: usize, start: Height) -> Cents {
        let start = start.to_usize();
        debug_assert!(start <= index, "price SMA window starts after its height");

        let current = u64::from(prefix_sum[index]);
        let previous = start
            .checked_sub(1)
            .map(|index| u64::from(prefix_sum[index]))
            .unwrap_or_default();
        let count = index - start + 1;

        Cents::new((current - previous) / count as u64)
    }

    fn try_for_each_value<E>(
        &self,
        from: usize,
        to: usize,
        mut each: impl FnMut(Cents) -> Result<(), E>,
    ) -> Result<(), E> {
        let prefix_sum = self.prefix_sum.snapshot();
        let to = to.min(self.window_starts.len()).min(prefix_sum.len());
        if from >= to {
            return Ok(());
        }

        for (offset, start) in self
            .window_starts
            .collect_range_dyn(from, to)
            .into_iter()
            .enumerate()
        {
            each(Self::average(&prefix_sum, from + offset, start))?;
        }
        Ok(())
    }

    fn for_each_value(&self, from: usize, to: usize, mut each: impl FnMut(Cents)) {
        let prefix_sum = self.prefix_sum.snapshot();
        let to = to.min(self.window_starts.len()).min(prefix_sum.len());
        if from >= to {
            return;
        }

        let mut index = from;
        self.window_starts
            .for_each_range_dyn_at(from, to, &mut |start| {
                each(Self::average(&prefix_sum, index, start));
                index += 1;
            });
    }
}

impl AnyVec for LazySmaVec {
    fn version(&self) -> Version {
        self.version + self.window_starts.version() + self.prefix_sum.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.window_starts.len().min(self.prefix_sum.len())
    }

    fn index_type_to_string(&self) -> &'static str {
        <Height as PrintableIndex>::to_string()
    }

    fn region_names(&self) -> Vec<String> {
        Vec::new()
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<Cents>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<Cents>()
    }
}

impl TypedVec for LazySmaVec {
    type I = Height;
    type T = Cents;
}

impl ReadableVec<Height, Cents> for LazySmaVec {
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<Cents>) {
        buf.reserve(to.min(self.len()).saturating_sub(from));
        self.for_each_value(from, to, |value| buf.push(value));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(Cents)) {
        self.for_each_value(from, to, each);
    }

    fn fold_range_at<B, F: FnMut(B, Cents) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: F,
    ) -> B {
        let mut acc = Some(init);
        self.for_each_value(from, to, |value| {
            acc = Some(fold(acc.take().unwrap(), value));
        });
        acc.unwrap()
    }

    fn try_fold_range_at<B, E, F: FnMut(B, Cents) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: F,
    ) -> Result<B, E> {
        let mut acc = Some(init);
        self.try_for_each_value(from, to, |value| {
            acc = Some(fold(acc.take().unwrap(), value)?);
            Ok(())
        })?;
        Ok(acc.unwrap())
    }

    fn collect_one_at(&self, index: usize) -> Option<Cents> {
        if index >= self.len() {
            return None;
        }
        let start = self.window_starts.collect_one_at(index)?;
        Some(Self::average(&self.prefix_sum.snapshot(), index, start))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<Cents>) {
        let len = self.len();
        let indices: Vec<_> = indices
            .iter()
            .copied()
            .filter(|index| *index < len)
            .collect();
        let starts = self.window_starts.read_sorted_at(&indices);
        let prefix_sum = self.prefix_sum.snapshot();

        out.reserve(indices.len());
        indices
            .into_iter()
            .zip(starts)
            .for_each(|(index, start)| out.push(Self::average(&prefix_sum, index, start)));
    }
}

#[cfg(test)]
mod tests {
    use vecdb::{
        AnyStoredVec, CachedReadableVec, CachedVec, Database, EagerVec, ImportableVec, PcoVec,
        ReadableCloneableVec, WritableVec,
    };

    use super::*;
    #[test]
    fn computes_rolling_average_from_one_shared_prefix_cache() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("brk-lazy-sma-{}-{suffix}", std::process::id()));
        let db = Database::open(&path).unwrap();

        let mut prices: EagerVec<PcoVec<Height, Cents>> =
            EagerVec::forced_import(&db, "prices", Version::ONE).unwrap();
        let mut starts: EagerVec<PcoVec<Height, Height>> =
            EagerVec::forced_import(&db, "starts", Version::ONE).unwrap();

        for value in [100, 200, 300, 400] {
            prices.push(Cents::new(value));
        }
        for value in [0, 0, 1, 2] {
            starts.push(Height::new(value));
        }
        prices.write().unwrap();
        starts.write().unwrap();

        let prices = CachedVec::wrap(prices);
        let prefix_sum = CachedVec::wrap(SmaPrefixSumVec::new(
            "prefix",
            Version::ONE,
            prices.read_only_cached_boxed_clone(),
        ));
        let sma = LazySmaVec::new(
            "sma",
            Version::ONE,
            starts.read_only_boxed_clone(),
            prefix_sum.cached_boxed_clone(),
        );

        assert_eq!(
            prefix_sum.snapshot().as_slice(),
            [
                StoredU64::new(100),
                StoredU64::new(300),
                StoredU64::new(600),
                StoredU64::new(1_000),
            ],
        );
        assert_eq!(
            sma.collect_range(Height::ZERO, Height::new(4)),
            [
                Cents::new(100),
                Cents::new(150),
                Cents::new(250),
                Cents::new(350),
            ],
        );
    }
}
