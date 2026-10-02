use brk_exit::Exit;
use brk_types::{Height, TxInIndex, TxOutIndex, Version};
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, AnyVec, BytesVec, Database, ImportableVec, ReadableVec, Stamp, WritableVec,
};

use super::{RANGE_LEN, build_ranges, range_bits, reset_incomplete};
use crate::spent::{compute::checkpoint, forced_import};

#[test]
fn packed_ranges_preserve_full_input_indexes_at_bit_boundaries() {
    for (input_end, expected_bits) in [
        (0, 26),
        (1, 26),
        (1_usize << 32, 26),
        (1_usize << 38, 26),
        ((1_usize << 38) + 1, 25),
    ] {
        let bits = range_bits(input_end, RANGE_LEN);
        assert_eq!(bits, expected_bits);
        let mask = (1_u64 << bits) - 1;
        for input in [
            0,
            (input_end / 2) as u64,
            input_end.saturating_sub(1) as u64,
        ] {
            for offset in [0, mask / 2, mask] {
                let record = (input << bits) | offset;
                assert_eq!(record >> bits, input);
                assert_eq!(record & mask, offset);
            }
        }
    }
}

#[test]
fn partitioned_build_matches_input_order_across_ranges_and_reopens() {
    let directory = tempdir().unwrap();
    let expected = {
        let db = Database::open(directory.path()).unwrap();
        let mut inputs =
            BytesVec::<TxInIndex, TxOutIndex>::forced_import(&db, "source", Version::ONE).unwrap();
        let mut expected = vec![TxInIndex::UNSPENT; 1_031];
        for input in 0..80_000 {
            // Fill one range beyond a scratch read buffer, then cover the rest.
            let outputs = if input < 70_000 { 64 } else { expected.len() };
            let output = (input * 137) % outputs;
            if input % 23 == 0 {
                inputs.push(TxOutIndex::COINBASE);
            } else {
                inputs.push(TxOutIndex::from(output));
                expected[output] = TxInIndex::from(input);
            }
        }
        // Excluded inputs must not change the requested prefix.
        inputs.push(TxOutIndex::ZERO);
        inputs.write().unwrap();
        let mut vecs = forced_import(&db, Version::ONE).unwrap();
        build_ranges(&mut vecs, &inputs, 80_000, expected.len(), 64, &Exit::new()).unwrap();
        assert_eq!(vecs.txin_index.stamp(), Stamp::default());
        assert_eq!(vecs.txin_index.collect(), expected);
        vecs.txin_index
            .stamped_write_maybe_with_changes(Stamp::new(100), false)
            .unwrap();
        vecs.txin_index.flush().unwrap();
        expected
    };
    let db = Database::open(directory.path()).unwrap();
    let mut vecs = forced_import(&db, Version::ONE).unwrap();
    reset_incomplete(&mut vecs).unwrap();
    assert_eq!(vecs.txin_index.collect(), expected);
    assert_eq!(vecs.txin_index.stamp(), Stamp::new(100));
    assert!(!directory.path().join("changes").exists());
}

#[test]
fn partial_build_is_discarded_and_final_block_can_be_replaced_after_reopen() {
    let directory = tempdir().unwrap();
    {
        let db = Database::open(directory.path()).unwrap();
        let mut vecs = forced_import(&db, Version::ONE).unwrap();
        // Simulate termination after writing an output range, before publishing
        // the baseline stamp. Its length does not represent a complete block.
        vecs.txin_index.push(TxInIndex::new(99));
        vecs.txin_index.flush().unwrap();
    }
    {
        let db = Database::open(directory.path()).unwrap();
        let mut vecs = forced_import(&db, Version::ONE).unwrap();
        reset_incomplete(&mut vecs).unwrap();
        assert!(vecs.txin_index.is_empty());
        let mut inputs =
            BytesVec::<TxInIndex, TxOutIndex>::forced_import(&db, "source", Version::ONE).unwrap();
        inputs.push(TxOutIndex::COINBASE);
        inputs.push(TxOutIndex::from(3_usize));
        inputs.write().unwrap();
        build_ranges(&mut vecs, &inputs, 2, 7, 2, &Exit::new()).unwrap();
        vecs.txin_index
            .stamped_write_maybe_with_changes(Stamp::new(10), false)
            .unwrap();
        vecs.txin_index.flush().unwrap();
        vecs.txin_index.update_at(0, TxInIndex::new(2)).unwrap();
        vecs.txin_index.push(TxInIndex::UNSPENT);
        checkpoint(&mut vecs, Height::new(10)).unwrap();
    }
    {
        let db = Database::open(directory.path()).unwrap();
        let mut vecs = forced_import(&db, Version::ONE).unwrap();
        assert_eq!(
            vecs.txin_index.rollback_before(Stamp::new(11)).unwrap(),
            Stamp::new(10)
        );
        assert_eq!(
            vecs.txin_index.collect(),
            [
                TxInIndex::UNSPENT,
                TxInIndex::UNSPENT,
                TxInIndex::UNSPENT,
                TxInIndex::new(1),
                TxInIndex::UNSPENT,
                TxInIndex::UNSPENT,
                TxInIndex::UNSPENT,
            ]
        );
        vecs.txin_index.update_at(1, TxInIndex::new(2)).unwrap();
        checkpoint(&mut vecs, Height::new(10)).unwrap();
    }
    let db = Database::open(directory.path()).unwrap();
    let vecs = forced_import(&db, Version::ONE).unwrap();
    assert_eq!(vecs.txin_index.len(), 7);
    assert_eq!(vecs.txin_index.collect_one_at(0), Some(TxInIndex::UNSPENT));
    assert_eq!(vecs.txin_index.collect_one_at(1), Some(TxInIndex::new(2)));
}
