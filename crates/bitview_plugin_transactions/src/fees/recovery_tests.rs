use super::{HeightMap, Sources};
use bitview_collections::Windows;
use bitview_vecs::{LazyWindowStartVec, import_cached};
use brk_exit::Exit;
use brk_types::{
    Height, Lengths, OutPoint, Sats, StoredU64, Timestamp, TxInIndex, TxIndex, TxOutIndex, Version,
    Weight,
};
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, AnyVec, Budgeted, BytesVec, Database, ImportableVec, OverflowVec, PcoVec,
    ReadableVec, WritableVec,
};

use crate::{fees::import, test_common};

struct Raw {
    block_txs: PcoVec<Height, TxIndex, Budgeted>,
    block_inputs: PcoVec<Height, TxInIndex, Budgeted>,
    block_outputs: PcoVec<Height, TxOutIndex, Budgeted>,
    tx_inputs: PcoVec<TxIndex, TxInIndex>,
    tx_outputs: BytesVec<TxIndex, TxOutIndex>,
    weights: PcoVec<TxIndex, Weight>,
    outpoints: PcoVec<TxInIndex, OutPoint>,
    values: OverflowVec<TxOutIndex, Sats>,
}
impl Raw {
    fn sources(&self) -> Sources<'_> {
        Sources {
            block_txs: &self.block_txs,
            block_inputs: &self.block_inputs,
            block_outputs: &self.block_outputs,
            tx_inputs: &self.tx_inputs,
            tx_outputs: &self.tx_outputs,
            weights: &self.weights,
            outpoints: &self.outpoints,
            values: &self.values,
        }
    }
}

#[test]
fn monetary_and_cpfp_groups_recover_independently_with_canonical_totals() {
    test_common::init_cache();
    let directory = tempdir().unwrap();
    let raw_db = Database::open(&directory.path().join("raw")).unwrap();
    let mut raw = Raw {
        block_txs: PcoVec::import(&raw_db, "first_tx_index", Version::ONE).unwrap(),
        block_inputs: PcoVec::import(&raw_db, "block_first_txin_index", Version::ONE).unwrap(),
        block_outputs: PcoVec::import(&raw_db, "block_first_txout_index", Version::ONE).unwrap(),
        tx_inputs: PcoVec::import(&raw_db, "first_txin_index", Version::ONE).unwrap(),
        tx_outputs: BytesVec::import(&raw_db, "first_txout_index", Version::ONE).unwrap(),
        weights: PcoVec::import(&raw_db, "weight", Version::ONE).unwrap(),
        outpoints: PcoVec::import(&raw_db, "outpoint", Version::ONE).unwrap(),
        values: OverflowVec::import(&raw_db, "value", Version::ONE).unwrap(),
    };
    for n in [0usize, 2, 4] {
        raw.block_txs.push(TxIndex::from(n));
        raw.block_inputs.push(TxInIndex::from(n));
        raw.block_outputs.push(TxOutIndex::from(n));
    }
    for n in 0usize..6 {
        raw.tx_inputs.push(TxInIndex::from(n));
        raw.tx_outputs.push(TxOutIndex::from(n));
        raw.weights.push(Weight::from(400u64));
        raw.outpoints.push(OutPoint::COINBASE);
        raw.values
            .push(Sats::new(if n % 2 == 0 { 5_000 } else { 900 }));
    }
    raw.block_txs.write().unwrap();
    raw.tx_inputs.write().unwrap();
    raw.tx_outputs.write().unwrap();
    raw.weights.write().unwrap();
    raw.block_inputs.write().unwrap();
    raw.block_outputs.write().unwrap();
    raw.outpoints.write().unwrap();
    raw.values.write().unwrap();
    let sources = Database::open(&directory.path().join("sources")).unwrap();
    let counts =
        test_common::stored::<Height, StoredU64>(&sources, "counts", [StoredU64::from(2u64); 3]);
    let timestamps = test_common::stored::<Height, Timestamp>(
        &sources,
        "timestamps",
        [0u32, 600, 1200].map(Timestamp::new),
    );
    let mut inputs =
        PcoVec::<TxInIndex, Sats>::import(&sources, "input_values", Version::ONE).unwrap();
    for _ in 0..3 {
        inputs.push(Sats::MAX);
        inputs.push(Sats::new(1_000));
    }
    inputs.write().unwrap();
    let map = HeightMap::from([0usize, 2, 4].map(TxIndex::from).to_vec());
    let indexes = test_common::indexes(&sources);
    let window = LazyWindowStartVec::days("window", Version::ONE, 1, &timestamps);
    let windows = Windows {
        _24h: &window,
        _1w: &window,
        _1m: &window,
        _1y: &window,
    };
    let path = directory.path().join("fees");
    let db = Database::open(&path).unwrap();
    let mut fees = import::forced_import(&db, Version::ONE, &indexes, &windows).unwrap();
    let mut volume = import_cached::<Height, Sats>(&db, "volume", Version::ONE).unwrap();
    let exit = Exit::new();
    let full = Lengths {
        height: Height::from(3usize),
        tx_index: TxIndex::from(6usize),
        ..Default::default()
    };
    fees.compute_fees(
        raw.sources(),
        full,
        &inputs,
        &map,
        &counts,
        &mut volume,
        &exit,
    )
    .unwrap();
    assert_eq!(fees.total.collect(), [Sats::new(100); 3]);
    assert_eq!(
        fees.fee.tx_index.collect(),
        [0u64, 100, 0, 100, 0, 100].map(Sats::new)
    );
    assert_eq!(volume.collect(), [1000u64, 2000, 3000].map(Sats::new));
    let monetary_version = fees.fee.tx_index.header().computed_version();
    let rates = fees.effective_fee_rate.tx_index.collect();

    // A CPFP-only source revision must not need resolved values or erase fees.
    inputs.truncate_if_needed_at(0).unwrap();
    inputs.write().unwrap();
    raw.outpoints
        .validate_computed_version_or_reset(Version::new(101))
        .unwrap();
    for _ in 0..6 {
        raw.outpoints.push(OutPoint::COINBASE);
    }
    raw.outpoints.write().unwrap();
    fees.compute_fees(
        raw.sources(),
        full,
        &inputs,
        &map,
        &counts,
        &mut volume,
        &exit,
    )
    .unwrap();
    assert_eq!(
        fees.fee.tx_index.header().computed_version(),
        monetary_version
    );
    assert_eq!(fees.fee.tx_index.len(), 6);
    assert_eq!(fees.effective_fee_rate.tx_index.collect(), rates);
    assert_eq!(volume.collect(), [1000u64, 2000, 3000].map(Sats::new));

    // Repair missing totals and a rate group ending inside a block on reopen.
    fees.total.truncate_if_needed_at(0).unwrap();
    fees.total.write().unwrap();
    fees.fee_rate.truncate_if_needed_at(3).unwrap();
    fees.fee_rate.write().unwrap();
    drop(fees);
    drop(volume);
    db.flush().unwrap();
    drop(db);
    let db = Database::open(&path).unwrap();
    let mut fees = import::forced_import(&db, Version::ONE, &indexes, &windows).unwrap();
    let mut volume = import_cached::<Height, Sats>(&db, "volume", Version::ONE).unwrap();
    fees.compute_fees(
        raw.sources(),
        full,
        &inputs,
        &map,
        &counts,
        &mut volume,
        &exit,
    )
    .unwrap();
    assert_eq!(fees.total.collect(), [Sats::new(100); 3]);
    assert_eq!(fees.fee_rate.len(), 6);
    assert_eq!(fees.effective_fee_rate.tx_index.collect(), rates);
    // An unavailable revised rate source leaves valid monetary facts intact.
    raw.weights
        .validate_computed_version_or_reset(Version::new(201))
        .unwrap();
    raw.weights.write().unwrap();
    fees.compute_fees(
        raw.sources(),
        full,
        &inputs,
        &map,
        &counts,
        &mut volume,
        &exit,
    )
    .unwrap();
    assert_eq!(fees.fee.tx_index.len(), 6);
    assert_eq!(fees.total.collect(), [Sats::new(100); 3]);
    assert_eq!(volume.collect(), [1000u64, 2000, 3000].map(Sats::new));
    assert_eq!(fees.fee_rate.len(), 0);
    assert_eq!(fees.effective_fee_rate.tx_index.len(), 0);
}
