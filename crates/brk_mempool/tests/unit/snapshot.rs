use bitcoin::{Txid as BitcoinTxid, hashes::Hash};
use brk_types::{Sats, VSize, Weight};
use smallvec::SmallVec;

use super::*;
use crate::test_support::fake_tx;

impl Snapshot {
    /// Test-only: stitch a snapshot from `(prefix, chunk_rate)` pairs
    /// without running the full builder.
    pub(crate) fn for_test_with_chunk_rates(entries: &[(TxidPrefix, FeeRate, Txid)]) -> Self {
        let mut prefix_to_idx = PrefixIndex::default();
        let mut txs = Vec::with_capacity(entries.len());
        for (i, (prefix, rate, txid)) in entries.iter().enumerate() {
            prefix_to_idx.insert(*prefix, TxIndex::from(i));
            txs.push(SnapTx {
                txid: *txid,
                fee: Sats::ZERO,
                vsize: VSize::from(0u64),
                weight: Weight::from(0u64),
                size: 0,
                chunk_rate: *rate,
                parents: SmallVec::new(),
                children: SmallVec::new(),
            });
        }
        Self {
            txs,
            blocks: vec![],
            block_stats: vec![],
            fees: RecommendedFees::default(),
            min_fee: FeeRate::default(),
            next_block_hash: NextBlockHash::ZERO,
            prefix_to_idx,
            template_transactions: Arc::from([]),
            content_revision: 0,
            template_missing: false,
        }
    }
}

fn snap_tx(seed: u8) -> SnapTx {
    let mut bytes = [0u8; 32];
    bytes[0] = seed;
    SnapTx {
        txid: Txid::from(BitcoinTxid::from_byte_array(bytes)),
        fee: Sats::from(1_234u64),
        vsize: VSize::from(100u64),
        weight: Weight::from(400u64),
        size: 100,
        chunk_rate: FeeRate::from((Sats::from(1_234u64), VSize::from(100u64))),
        parents: SmallVec::new(),
        children: SmallVec::new(),
    }
}

#[test]
fn next_block_hash_is_deterministic_across_runs() {
    assert_eq!(template_hash(&[1, 2, 3]), template_hash(&[1, 2, 3]));
}

fn template_hash(seeds: &[u8]) -> NextBlockHash {
    let mut snapshot = Snapshot::default();
    let bodies = seeds
        .iter()
        .map(|seed| Arc::new(fake_tx(*seed, &[], &[])))
        .collect();
    snapshot.set_template(bodies, 0, false);
    snapshot.next_block_hash
}

#[test]
fn next_block_hash_changes_with_block0_membership() {
    assert_ne!(template_hash(&[1, 2]), template_hash(&[1, 2, 3]));
}

#[test]
fn next_block_hash_changes_with_block0_order() {
    assert_ne!(template_hash(&[1, 2, 3]), template_hash(&[3, 2, 1]));
}

#[test]
fn empty_blocks_hash_is_zero() {
    assert_eq!(Snapshot::default().next_block_hash, NextBlockHash::ZERO);
}

#[test]
fn full_txid_lookup_rejects_prefix_collision() {
    let indexed = snap_tx(1).txid;
    let mut bytes = [0u8; 32];
    bytes[0] = 1;
    bytes[8] = 1;
    let collision = Txid::from(BitcoinTxid::from_byte_array(bytes));
    assert_eq!(TxidPrefix::from(&indexed), TxidPrefix::from(&collision));

    let snapshot = Snapshot::for_test_with_chunk_rates(&[(
        TxidPrefix::from(&indexed),
        FeeRate::default(),
        indexed,
    )]);
    assert_eq!(snapshot.idx_of_txid(&indexed), Some(TxIndex::from(0usize)));
    assert_eq!(snapshot.idx_of_txid(&collision), None);
    assert_eq!(snapshot.chunk_rate_for(&indexed), Some(FeeRate::default()));
    assert_eq!(snapshot.chunk_rate_for(&collision), None);
}
