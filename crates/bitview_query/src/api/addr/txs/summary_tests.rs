use std::{marker::PhantomData, sync::atomic::{AtomicUsize, Ordering}};

use bitcoin::ScriptBuf;
use bitview_primitives::{TxInIndex, TypeIndex};
use brk_types::{OutputType, Sats, TxIndex};
use vecdb::{AnyVec, ReadableVec, VecIndex, VecValue, Version};

use super::{add_script_values, transaction_ranges};

struct TestVec<I, T> {
    values: Vec<T>,
    reads: AtomicUsize,
    index: PhantomData<I>,
}

impl<I, T> TestVec<I, T> {
    fn new(values: Vec<T>) -> Self {
        Self { values, reads: AtomicUsize::new(0), index: PhantomData }
    }
}

impl<I: VecIndex, T: VecValue> AnyVec for TestVec<I, T> {
    fn version(&self) -> Version { Version::ZERO }
    fn name(&self) -> &str { "summary-test" }
    fn len(&self) -> usize { self.values.len() }
    fn is_mutable(&self) -> bool { false }
    vecdb::any_vec_type_methods!(I, T);
}

impl<I: VecIndex, T: VecValue> ReadableVec<I, T> for TestVec<I, T> {
    fn cursor_chunk_size(&self) -> usize { 2 }
    fn read_into_at(&self, from: usize, to: usize, out: &mut Vec<T>) {
        self.reads.fetch_add(1, Ordering::Relaxed);
        out.extend_from_slice(&self.values[from..to.min(self.len())]);
    }
    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(T)) {
        self.values[from..to.min(self.len())].iter().cloned().for_each(f);
    }
    fn try_fold_range_at<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self, from: usize, to: usize, init: B, f: F,
    ) -> Result<B, E> {
        self.values[from..to.min(self.len())].iter().cloned().try_fold(init, f)
    }
}

#[test]
fn summary_boundaries_share_chunks_and_end_at_publication() {
    // The fifth start is already written but unpublished. It must not end transaction 3.
    let starts = TestVec::<TxIndex, _>::new(vec![0_usize, 2, 5, 6, 900]);
    let ranges = transaction_ranges(&starts, &[0, 2, 3], TxIndex::new(4), 8).unwrap();
    assert_eq!(ranges, [0..2, 5..6, 6..8]);
    assert_eq!(starts.reads.load(Ordering::Relaxed), 2);
    assert!(transaction_ranges(&starts, &[4], TxIndex::new(4), 8).is_err());
    assert_eq!(starts.reads.load(Ordering::Relaxed), 2);
    let bad = TestVec::<TxIndex, _>::new(vec![0_usize, 9]);
    assert!(transaction_ranges(&bad, &[0], TxIndex::new(2), 8).is_err());
}

#[test]
fn summary_net_matches_full_script_events_including_zero_and_type_collisions() {
    let scripts = [
        ScriptBuf::from_bytes([vec![0, 20], vec![1; 20]].concat()),
        ScriptBuf::from_bytes([vec![0, 20], vec![2; 20]].concat()),
        ScriptBuf::from_bytes([vec![0x51, 32], vec![3; 32]].concat()),
    ];
    // Separate address families deliberately reuse numeric index 7.
    let identities = [(OutputType::P2WPKH, TypeIndex::new(7)),
        (OutputType::P2WPKH, TypeIndex::new(8)), (OutputType::P2TR, TypeIndex::new(7))];
    // Full script/value events are the oracle, independent of the column match operation.
    let events = [
        (vec![(None, 0_u64)], vec![(0, 5_000_000_000_u64)]),
        (vec![(Some(0), 50), (Some(0), 20), (Some(2), 900)], vec![(0, 40), (0, 10), (1, 100)]),
        (vec![(Some(0), 7)], vec![(0, 7), (2, 700)]),
        (vec![(Some(0), 11)], vec![(1, 11)]),
    ];
    let mut input_ranges = Vec::new();
    let mut output_ranges = Vec::new();
    let (mut input_types, mut input_ids, mut input_values) = (Vec::new(), Vec::new(), Vec::new());
    let (mut output_types, mut output_ids, mut output_values) = (Vec::new(), Vec::new(), Vec::new());
    for (inputs, outputs) in &events {
        let start = input_types.len();
        for &(script, value) in inputs {
            let (kind, index) = script.map(|script| identities[script])
                .map_or((OutputType::UnknownScript, None), |(kind, index)| (kind, Some(index)));
            input_types.push(kind); input_ids.push(index); input_values.push(Sats::from(value));
        }
        input_ranges.push(start..input_types.len());
        let start = output_types.len();
        for &(script, value) in outputs {
            let (kind, index) = identities[script];
            output_types.push(kind); output_ids.push(index); output_values.push(Sats::from(value));
        }
        output_ranges.push(start..output_types.len());
    }
    let input_types = TestVec::<TxInIndex, _>::new(input_types);
    let input_ids = TestVec::<TxInIndex, _>::new(input_ids);
    let input_values = TestVec::<TxInIndex, _>::new(input_values);
    let output_types = TestVec::<usize, _>::new(output_types);
    let output_ids = TestVec::<usize, _>::new(output_ids);
    let output_values = TestVec::<usize, _>::new(output_values);
    for (target, &identity) in identities.iter().enumerate() {
        let expected: Vec<i64> = events.iter().map(|(inputs, outputs)| {
            let funded: i128 = outputs.iter()
                .filter(|(id, _)| scripts[*id] == scripts[target]).map(|(_, value)| i128::from(*value)).sum();
            let spent: i128 = inputs.iter()
                .filter(|(id, _)| id.is_some_and(|id| scripts[id] == scripts[target]))
                .map(|(_, value)| i128::from(*value)).sum();
            (funded - spent).try_into().unwrap()
        }).collect();
        let mut actual = vec![0; events.len()];
        add_script_values::<_, _, true>(&input_ranges, identity,
            (&input_types, &input_ids, &input_values), &mut actual, || Ok(())).unwrap();
        add_script_values::<_, _, false>(&output_ranges, identity,
            (&output_types, &output_ids, &output_values), &mut actual, || Ok(())).unwrap();
        assert_eq!(actual, expected);
        if target == 0 { assert_eq!(actual, [5_000_000_000, -20, 0, -11]); }
    }
}
