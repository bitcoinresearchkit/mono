use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering::Relaxed},
};

use bitview_vecs::RangeMapVec;
use brk_types::{Date, Day1, Height, Timestamp, Version};
use rangeindex::SharedRangeMap;
use vecdb::{AnyVec, ReadableVec};

use super::{DatedResolutionVecs, ResolutionVecs};

#[derive(Clone)]
struct Periods {
    values: RangeMapVec<Height, Day1>,
    mapping: SharedRangeMap<Day1, Height>,
    reads: Arc<AtomicUsize>,
}

impl Periods {
    fn new(values: &[usize]) -> Self {
        let mapping = SharedRangeMap::new(values.iter().copied().map(Day1::from).collect());
        Self {
            values: RangeMapVec::new("periods", Version::ONE, mapping.clone()),
            mapping,
            reads: Arc::default(),
        }
    }

    fn update(&self, from: usize, values: &[usize]) {
        self.mapping
            .update_at(from, values.iter().copied().map(Day1::from));
    }

    fn take_reads(&self) -> usize {
        self.reads.swap(0, Relaxed)
    }
}

impl AnyVec for Periods {
    fn version(&self) -> Version {
        self.values.version()
    }
    fn name(&self) -> &str {
        self.values.name()
    }
    fn len(&self) -> usize {
        self.values.len()
    }
    fn index_type_to_string(&self) -> &'static str {
        self.values.index_type_to_string()
    }
    fn value_type_to_size_of(&self) -> usize {
        self.values.value_type_to_size_of()
    }
    fn value_type_to_string(&self) -> &'static str {
        self.values.value_type_to_string()
    }
    fn region_names(&self) -> Vec<String> {
        Vec::new()
    }
}

impl ReadableVec<Height, Day1> for Periods {
    fn read_into_at(&self, from: usize, to: usize, out: &mut Vec<Day1>) {
        let before = out.len();
        self.values.read_into_at(from, to, out);
        self.reads.fetch_add(out.len() - before, Relaxed);
    }
    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(Day1)) {
        self.collect_range_at(from, to).into_iter().for_each(each);
    }
    fn fold_range_at<B, F: FnMut(B, Day1) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: F,
    ) -> B {
        self.collect_range_at(from, to).into_iter().fold(init, fold)
    }
    fn try_fold_range_at<B, E, F: FnMut(B, Day1) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: F,
    ) -> Result<B, E> {
        self.collect_range_at(from, to)
            .into_iter()
            .try_fold(init, fold)
    }
}

fn firsts(vec: &ResolutionVecs<Day1>) -> Vec<usize> {
    vec.first_height
        .collect()
        .into_iter()
        .map(usize::from)
        .collect()
}

#[test]
fn resident_boundaries_replay_only_the_affected_height_suffix() {
    let source = Periods::new(&[]);
    let mut resolution = ResolutionVecs::new(&source);
    let reader = resolution.first_height.clone();
    let reverse = resolution.height_lookup();
    assert!(firsts(&resolution).is_empty());
    assert!(reverse.collect().is_empty());

    source.update(0, &[2, 2, 4, 4, 7]);
    resolution.update(Height::ZERO);
    assert_eq!(firsts(&resolution), [0, 0, 0, 2, 2, 4, 4, 4]);
    assert_eq!(reverse.collect(), [2, 2, 4, 4, 7].map(Day1::from));
    assert_eq!(source.take_reads(), 5);

    // Reads and clones use the resident table, never the source mapping.
    assert_eq!(reader.collect(), resolution.first_height.collect());
    assert_eq!(reader.collect_one_at(4), Some(Height::new(2)));
    assert_eq!(source.take_reads(), 0);

    source.update(5, &[7]);
    resolution.update(Height::new(5));
    assert_eq!(firsts(&resolution), [0, 0, 0, 2, 2, 4, 4, 4]);
    assert_eq!(reverse.collect(), [2, 2, 4, 4, 7, 7].map(Day1::from));
    assert_eq!(source.take_reads(), 2);

    source.update(6, &[9]);
    resolution.update(Height::new(6));
    assert_eq!(firsts(&resolution), [0, 0, 0, 2, 2, 4, 4, 4, 6, 6]);
    assert_eq!(reverse.collect(), [2, 2, 4, 4, 7, 7, 9].map(Day1::from));
    assert_eq!(source.take_reads(), 2);

    // Same-length reorg crosses periods and removes obsolete gaps.
    source.update(4, &[5, 5, 6]);
    resolution.update(Height::new(4));
    assert_eq!(firsts(&resolution), [0, 0, 0, 2, 2, 4, 6]);
    assert_eq!(reverse.collect(), [2, 2, 4, 4, 5, 5, 6].map(Day1::from));
    assert_eq!(source.take_reads(), 4);
    assert_eq!(reader.collect(), resolution.first_height.collect());

    // Shortening a period preserves the surviving block's boundary.
    source.update(3, &[]);
    resolution.update(Height::new(3));
    assert_eq!(firsts(&resolution), [0, 0, 0, 2, 2]);
    assert_eq!(reverse.collect(), [2, 2, 4].map(Day1::from));
    assert_eq!(reverse.collect_one_at(3), None);
    assert_eq!(source.take_reads(), 1);
    assert_eq!(firsts(&ResolutionVecs::new(&source)), firsts(&resolution));

    source.update(0, &[]);
    resolution.update(Height::ZERO);
    assert!(reader.collect().is_empty());
    assert!(reverse.collect().is_empty());
}

#[test]
fn dates_follow_the_existing_period_or_first_timestamp_contract() {
    let periods = Periods::new(&[2, 2, 4, 4]);
    let timestamp = |day: usize| Timestamp::from(Date::from(Day1::from(day)));
    let timestamp_source = SharedRangeMap::new([2, 3, 8, 9].map(timestamp).to_vec());
    let timestamps =
        RangeMapVec::<Height, Timestamp>::new("timestamps", Version::TWO, timestamp_source.clone());
    let mut calendar = DatedResolutionVecs::from_period_date(&periods);
    let mut first = DatedResolutionVecs::from_first_timestamp(&periods, &timestamps);
    let first_reader = first.date.clone();
    assert_eq!(
        calendar.date.collect(),
        (0..5)
            .map(|day| Date::from(Day1::from(day)))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        first.date.collect(),
        [2, 2, 2, 8, 8].map(|day| Date::from(Day1::from(day)))
    );
    assert_eq!(first.date.version(), Version::TWO);

    // A replaced first block changes its date even if its period stays the same.
    timestamp_source.update_at(2, [11, 12].map(timestamp));
    first.update(Height::new(2));
    assert_eq!(
        first_reader.collect(),
        [2, 2, 2, 11, 11].map(|day| Date::from(Day1::from(day)))
    );

    periods.update(2, &[3]);
    timestamp_source.update_at(2, [timestamp(10)]);
    calendar.update(Height::new(2));
    first.update(Height::new(2));
    assert_eq!(calendar.date.len(), 4);
    assert_eq!(
        first_reader.collect(),
        [2, 2, 2, 10].map(|day| Date::from(Day1::from(day)))
    );
    let reopened = DatedResolutionVecs::from_first_timestamp(&periods, &timestamps);
    assert_eq!(reopened.date.collect(), first_reader.collect());
    assert_eq!(
        reopened.first_height.collect(),
        first.first_height.collect()
    );
}

#[test]
fn reconstruction_matches_a_reference_across_chunks_and_partial_bootstrap() {
    let values: Vec<_> = (0..20_001).map(|height| height / 3 * 2 + 2).collect();
    let source = Periods::new(&values[..100]);
    let mut resolution = ResolutionVecs::new(&source);
    source.update(100, &values[100..]);
    // An upstream cursor ahead of this mapping must not skip unprocessed blocks.
    resolution.update(Height::from(values.len()));
    let expected: Vec<_> = (0..=values[values.len() - 1])
        .map(|period| values.partition_point(|&value| value < period))
        .collect();
    assert_eq!(firsts(&resolution), expected);
    source.take_reads();
    resolution.update(Height::from(values.len()));
    assert_eq!(source.take_reads(), 0);
}
