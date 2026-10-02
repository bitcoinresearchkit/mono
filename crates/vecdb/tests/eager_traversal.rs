use std::{
    any,
    sync::atomic::{AtomicUsize, Ordering},
};

use brk_exit::Exit;
use tempfile::tempdir;
use vecdb::{
    AnyVec, BytesVec, BytesVecValue, Database, EagerVec, ImportableVec, ReadableVec,
    Result as VecdbResult, VecValue, Version,
};

struct CountingSource<T> {
    values: Vec<T>,
    version: Version,
    reads: AtomicUsize,
}

impl<T> CountingSource<T> {
    fn new(values: Vec<T>) -> Self {
        Self {
            values,
            version: Version::ONE,
            reads: AtomicUsize::new(0),
        }
    }
}

impl<T: VecValue> AnyVec for CountingSource<T> {
    fn version(&self) -> Version {
        self.version
    }
    fn name(&self) -> &str {
        "source"
    }
    fn len(&self) -> usize {
        self.values.len()
    }
    fn index_type_to_string(&self) -> &'static str {
        "usize"
    }
    fn region_names(&self) -> Vec<String> {
        vec![]
    }
    fn value_type_to_size_of(&self) -> usize {
        size_of::<T>()
    }
    fn value_type_to_string(&self) -> &'static str {
        any::type_name::<T>()
    }
}

impl<T: VecValue> ReadableVec<usize, T> for CountingSource<T> {
    fn read_into_at(&self, from: usize, to: usize, out: &mut Vec<T>) {
        self.fold_range_at(from, to, (), |(), value| out.push(value));
    }
    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        self.fold_range_at(from, to, (), |(), value| f(value));
    }
    fn fold_range_at<B, F: FnMut(B, T) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut f: F,
    ) -> B {
        let end = to.min(self.len());
        self.values[from.min(end)..end]
            .iter()
            .fold(init, |acc, value| {
                self.reads.fetch_add(1, Ordering::Relaxed);
                f(acc, value.clone())
            })
    }
    fn try_fold_range_at<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut f: F,
    ) -> Result<B, E> {
        let end = to.min(self.len());
        self.values[from.min(end)..end]
            .iter()
            .try_fold(init, |acc, value| {
                self.reads.fetch_add(1, Ordering::Relaxed);
                f(acc, value.clone())
            })
    }
}

fn lifecycle<T: VecValue, O: BytesVecValue + PartialEq>(
    values: Vec<T>,
    expected: &[O],
    mut compute: impl FnMut(
        &mut EagerVec<BytesVec<usize, O>>,
        usize,
        &CountingSource<T>,
        &Exit,
    ) -> VecdbResult<()>,
) -> VecdbResult<()> {
    let directory = tempdir()?;
    let db = Database::open(directory.path())?;
    let exit = Exit::new();
    let mut source = CountingSource::new(values);
    let mut output = EagerVec::import(&db, "output", Version::ONE)?;
    let split = expected.len().min(3);
    let tail = source.values.split_off(split);
    compute(&mut output, 0, &source, &exit)?;
    assert_eq!(output.collect(), expected[..split]);
    source.values.extend(tail);
    for from in [split, expected.len(), expected.len() + 4, 1] {
        source.reads.store(0, Ordering::Relaxed);
        compute(&mut output, from, &source, &exit)?;
        assert_eq!(output.collect(), expected);
        if from >= expected.len() {
            assert_eq!(source.reads.load(Ordering::Relaxed), 0);
        }
    }
    drop(output);
    let mut output = EagerVec::import(&db, "output", Version::ONE)?;
    compute(&mut output, 4, &source, &exit)?;
    assert_eq!(output.collect(), expected);
    source.version = Version::TWO;
    source.reads.store(0, Ordering::Relaxed);
    compute(&mut output, expected.len(), &source, &exit)?;
    assert_eq!(output.collect(), expected);
    assert!(source.reads.load(Ordering::Relaxed) > 0);
    Ok(())
}

#[test]
fn sma_preserves_resume_and_version_reset() -> VecdbResult<()> {
    // Integer means let the reference sum windows without repeating the recurrence.
    let values = vec![3_f32, 9., 0., 12., 6., 3., 9., 6.];
    let sma: Vec<_> = (0..values.len())
        .map(|i| {
            let window = &values[i.saturating_sub(2)..=i];
            window.iter().sum::<f32>() / window.len() as f32
        })
        .collect();
    lifecycle(values, &sma, |out, from, source, exit| {
        out.compute_sma(from, source, 3, exit, None)
    })?;
    Ok(())
}
