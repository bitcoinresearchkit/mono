use std::{convert::Infallible, sync::Arc};

use brk_types::{Height, StoredU16, StoredU64};
use parking_lot::RwLock;
use vecdb::{
    AnyVec, CachedReadableVec, PrintableIndex, ReadableVec, TypedVec, VecIndex, Version,
    short_type_name,
};

const CHECKPOINT_INTERVAL: usize = 256;

pub struct CachedBlockCountReader {
    block: Box<dyn CachedReadableVec<Height, StoredU16>>,
    checkpoints: Arc<RwLock<Checkpoints>>,
}

struct Checkpoints {
    block: Arc<[StoredU16]>,
    cumulative: Arc<[u64]>,
}

impl CachedBlockCountReader {
    pub(crate) fn new(block: Box<dyn CachedReadableVec<Height, StoredU16>>) -> Self {
        Self {
            block,
            checkpoints: Arc::new(RwLock::new(Checkpoints {
                block: Arc::from([]),
                cumulative: Arc::from([0]),
            })),
        }
    }

    pub(crate) fn clear(&self) {
        self.block.clear();
    }

    pub fn cumulative_at(&self, index: usize) -> Option<StoredU64> {
        let (block, checkpoints) = self.snapshot();
        (index < block.len())
            .then(|| StoredU64::from(Self::sum_before(&block, &checkpoints, index + 1)))
    }

    pub fn for_each_cumulative(&self, from: usize, to: usize, mut each: impl FnMut(StoredU64)) {
        self.try_fold_cumulative(from, to, (), |(), value| {
            each(value);
            Ok::<_, Infallible>(())
        })
        .unwrap();
    }

    pub fn for_each_rolling_sum(
        &self,
        from: usize,
        starts: &[Height],
        mut each: impl FnMut(StoredU64),
    ) {
        let (block, checkpoints) = self.snapshot();
        let to = (from + starts.len()).min(block.len());
        if from >= to {
            return;
        }

        let starts = &starts[..to - from];
        let mut start = starts[0].to_usize();
        let mut cumulative = Self::sum_before(&block, &checkpoints, from);
        let mut before_start = Self::sum_before(&block, &checkpoints, start);

        for (offset, next_start) in starts.iter().enumerate() {
            let next_start = next_start.to_usize();
            debug_assert!(next_start >= start);
            debug_assert!(next_start <= from + offset);

            if next_start >= start {
                for value in &block[start..next_start] {
                    before_start += Self::as_u64(value);
                }
            } else {
                before_start = Self::sum_before(&block, &checkpoints, next_start);
            }
            start = next_start;

            cumulative += Self::as_u64(&block[from + offset]);
            each(StoredU64::from(cumulative - before_start));
        }
    }

    pub fn rolling_sum_at(&self, start: usize, end: usize) -> Option<StoredU64> {
        let (block, checkpoints) = self.snapshot();
        if start > end || end >= block.len() {
            return None;
        }

        Some(StoredU64::from(
            Self::sum_before(&block, &checkpoints, end + 1)
                - Self::sum_before(&block, &checkpoints, start),
        ))
    }

    fn snapshot(&self) -> (Arc<[StoredU16]>, Arc<[u64]>) {
        let block = self.block.cached();

        {
            let checkpoints = self.checkpoints.read();
            if Arc::ptr_eq(&checkpoints.block, &block) {
                return (block, checkpoints.cumulative.clone());
            }
        }

        let cumulative = Self::build_checkpoints(&block);
        let mut checkpoints = self.checkpoints.write();
        if !Arc::ptr_eq(&checkpoints.block, &block) {
            checkpoints.block = block.clone();
            checkpoints.cumulative = cumulative;
        }

        (block, checkpoints.cumulative.clone())
    }

    fn try_fold_cumulative<B, E>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: impl FnMut(B, StoredU64) -> Result<B, E>,
    ) -> Result<B, E> {
        let (block, checkpoints) = self.snapshot();
        let to = to.min(block.len());
        if from >= to {
            return Ok(init);
        }

        let mut accumulator = init;
        let mut cumulative = Self::sum_before(&block, &checkpoints, from);

        for value in &block[from..to] {
            cumulative += Self::as_u64(value);
            accumulator = fold(accumulator, StoredU64::from(cumulative))?;
        }

        Ok(accumulator)
    }

    fn build_checkpoints(block: &[StoredU16]) -> Arc<[u64]> {
        let mut checkpoints = Vec::with_capacity(block.len() / CHECKPOINT_INTERVAL + 1);
        let mut cumulative = 0;
        checkpoints.push(cumulative);

        for (index, value) in block.iter().enumerate() {
            cumulative += Self::as_u64(value);
            if (index + 1) % CHECKPOINT_INTERVAL == 0 {
                checkpoints.push(cumulative);
            }
        }

        checkpoints.into()
    }

    #[inline(always)]
    fn sum_before(block: &[StoredU16], checkpoints: &[u64], end: usize) -> u64 {
        let end = end.min(block.len());
        let checkpoint = end / CHECKPOINT_INTERVAL;
        let from = checkpoint * CHECKPOINT_INTERVAL;
        checkpoints[checkpoint] + block[from..end].iter().map(Self::as_u64).sum::<u64>()
    }

    #[inline(always)]
    fn as_u64(value: &StoredU16) -> u64 {
        u64::from(**value)
    }
}

impl Clone for CachedBlockCountReader {
    fn clone(&self) -> Self {
        Self {
            block: self.block.cached_boxed_clone(),
            checkpoints: self.checkpoints.clone(),
        }
    }
}

impl AnyVec for CachedBlockCountReader {
    fn version(&self) -> Version {
        self.block.version()
    }

    fn name(&self) -> &str {
        self.block.name()
    }

    fn len(&self) -> usize {
        self.block.len()
    }

    fn index_type_to_string(&self) -> &'static str {
        <Height as PrintableIndex>::to_string()
    }

    fn region_names(&self) -> Vec<String> {
        Vec::new()
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<StoredU64>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<StoredU64>()
    }
}

impl TypedVec for CachedBlockCountReader {
    type I = Height;
    type T = StoredU64;
}

impl ReadableVec<Height, StoredU64> for CachedBlockCountReader {
    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<StoredU64>) {
        buf.reserve(to.saturating_sub(from));
        self.for_each_cumulative(from, to, |value| buf.push(value));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(StoredU64)) {
        self.for_each_cumulative(from, to, each);
    }

    fn fold_range_at<B, F: FnMut(B, StoredU64) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: F,
    ) -> B {
        self.try_fold_cumulative(from, to, init, |accumulator, value| {
            Ok::<_, Infallible>(fold(accumulator, value))
        })
        .unwrap()
    }

    fn try_fold_range_at<B, E, F: FnMut(B, StoredU64) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: F,
    ) -> Result<B, E> {
        self.try_fold_cumulative(from, to, init, fold)
    }

    fn collect_one_at(&self, index: usize) -> Option<StoredU64> {
        self.cumulative_at(index)
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<StoredU64>) {
        let (block, checkpoints) = self.snapshot();
        out.reserve(indices.len());
        indices
            .iter()
            .take_while(|&&index| index < block.len())
            .for_each(|&index| {
                out.push(StoredU64::from(Self::sum_before(
                    &block,
                    &checkpoints,
                    index + 1,
                )));
            });
    }
}

#[cfg(test)]
mod tests {
    use brk_types::{Height, StoredU16, StoredU64, Version};
    use vecdb::{AnyStoredVec, CachedVec, Database, EagerVec, ImportableVec, PcoVec, WritableVec};

    use super::*;

    #[test]
    fn reconstructs_cumulative_and_rolling_counts() {
        let path =
            std::env::temp_dir().join(format!("brk-cached-block-count-{}", std::process::id()));
        let db = Database::open(&path).unwrap();
        let mut block: EagerVec<PcoVec<Height, StoredU16>> =
            EagerVec::forced_import(&db, "count", Version::ONE).unwrap();

        let mut expected = Vec::new();
        let mut total = 0_u64;
        for index in 0..600 {
            let value = (index % 7) as u16;
            total += u64::from(value);
            expected.push(StoredU64::from(total));
            block.push(StoredU16::new(value));
        }
        block.write().unwrap();

        let mut block = CachedVec::wrap(block);
        let count = CachedBlockCountReader::new(block.read_only_cached_boxed_clone());

        assert_eq!(count.cumulative_at(599), Some(expected[599]));

        let mut reconstructed = Vec::new();
        count.for_each_cumulative(250, 270, |value| reconstructed.push(value));
        assert_eq!(reconstructed, expected[250..270]);

        let starts = (250..270)
            .map(|index| Height::new(index - 10))
            .collect::<Vec<_>>();
        let mut rolling = Vec::new();
        count.for_each_rolling_sum(250, &starts, |value| rolling.push(value));
        let expected = (250..270)
            .map(|index| {
                let current = expected[index];
                let previous = expected[index - 11];
                current - previous
            })
            .collect::<Vec<_>>();
        assert_eq!(rolling, expected);

        block.inner.truncate_if_needed_at(0).unwrap();
        for _ in 0..600 {
            block.inner.push(StoredU16::new(1));
        }
        block.inner.write().unwrap();
        block.clear();

        assert_eq!(count.cumulative_at(599), Some(StoredU64::from(600_u64)));

        drop(count);
        drop(block);
        drop(db);
        std::fs::remove_dir_all(path).unwrap();
    }
}
