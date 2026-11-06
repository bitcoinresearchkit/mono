use std::sync::Arc;

use bitcoin::hashes::Hash;
use bitcoin::{ScriptBuf, Witness};
use brk_cpfp::Cluster;
use brk_types::{
    BlockHash, CpfpClusterTxIndex, FeeRate, MempoolEntryInfo, Sats, SigOps, Timestamp, Transaction,
    TxIn, TxStatus, Txid, VSize, Vout, Weight,
};
use rapidhash::RapidHashSet;
use serde_json::{Value, to_value};
use smallvec::SmallVec;

use crate::{
    MempoolBlockSource, ReadOnlyState, State,
    snapshot::Rebuilder,
    snapshot::{
        SnapTx, Snapshot, TxIndex,
        builder::KeyIndex,
        cluster,
        partition::{Chunks, partition},
        rebuilder::withhold,
    },
    steps::{Confirmed, applier, preparer},
    stores::tx_store::TxRecord,
};

/// Random pools (chains, fans, sigop-heavy transactions), the last transaction a chunk no block
/// can hold unless the pool fits, packed after a block 0 that took an ancestor-closed prefix: each
/// member once, after its parents, every block but the last within Core's budgets, the oversized
/// chunk in the last, which then holds more than a block, and packing ends whatever is left (a
/// remainder of the oversized chunk alone included).
#[test]
fn projected_blocks_follow_dependencies_and_budgets() {
    for (n, taken, fits) in [
        (30_000, 2_000, false),
        (60, 10, false),
        (60, 10, true),
        (1, 0, false),
    ] {
        let mut txs = pool(n);
        if fits {
            let last = &mut txs[n - 1];
            last.weight = Weight::from(1_000u64);
            last.vsize = VSize::from(last.weight);
        }
        let excluded: Vec<bool> = (0..n).map(|i| i < taken).collect();
        let projection = partition(&txs, &chunks(&txs), &excluded, 7);
        let blocks = &projection.blocks;
        assert!(blocks.len() <= 7);
        assert_eq!(
            projection.overflow, !fits,
            "the last block holds more than a block"
        );
        let mut placed = excluded.clone();
        for (b, block) in blocks.iter().enumerate() {
            let (mut weight, mut sigops) = (Weight::default(), SigOps::ZERO);
            for &(m, _) in block {
                let t = &txs[m.as_usize()];
                assert!(!placed[m.as_usize()], "a transaction in two blocks");
                assert!(
                    t.parents.iter().all(|p| placed[p.as_usize()]),
                    "before a parent"
                );
                placed[m.as_usize()] = true;
                weight += t.weight;
                sigops += t.sigops;
            }
            if b + 1 < blocks.len() {
                assert!(weight < Weight::from(3_992_000u64) && sigops < SigOps::new(79_600));
            }
        }
        assert!(placed.iter().all(|&p| p), "a transaction left out");
        let last = blocks.last().expect("blocks");
        let oversized = last.iter().any(|&(m, _)| m.as_usize() == n - 1);
        assert!(fits || oversized, "the oversized chunk waits for the last");
    }
}

/// `n` transactions, each parent earlier than its children; the last one oversized.
fn pool(n: usize) -> Vec<SnapTx> {
    let mut state = 0x2545_F491_4F6C_DD1Du64;
    let mut next = |bound: u64| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state % bound
    };
    let mut txs: Vec<SnapTx> = (0..n)
        .map(|i| {
            let mut bytes = [0u8; 32];
            bytes[..8].copy_from_slice(&next(u64::MAX).to_le_bytes());
            let weight = Weight::from(400 + next(4_000));
            let sigops = SigOps::new(if next(50) == 0 { 2_000 } else { next(8) });
            let mut parents: SmallVec<[TxIndex; 2]> = SmallVec::new();
            if i > 0 && next(3) == 0 {
                parents.push(TxIndex::from(i - 1 - next(i.min(20) as u64) as usize));
            }
            let fee = Sats::from(next(40_000));
            SnapTx {
                txid: Txid::from(bitcoin::Txid::from_byte_array(bytes)),
                fee,
                modified: u64::from(fee) as i64,
                vsize: sigops.adjust_vsize(VSize::from(weight)),
                weight,
                size: 0,
                sigops,
                chunk_rate: FeeRate::default(),
                parents,
                children: SmallVec::new(),
            }
        })
        .collect();
    let last = &mut txs[n - 1];
    last.weight = Weight::from(3_995_000u64);
    last.vsize = VSize::from(last.weight);
    last.fee = Sats::from(100_000_000u64);
    last.modified = 100_000_000;
    for i in 0..n {
        for p in txs[i].parents.clone() {
            txs[p.as_usize()].children.push(TxIndex::from(i));
        }
    }
    txs
}

/// A CPFP answer read from the snapshot's linearization is the one a fresh linearization gives:
/// the same cluster, chunks and effective rate, for every member of random pools.
#[test]
fn cpfp_answers_match_a_fresh_linearization() {
    let txs = pool(2_000);
    let snapshot = Snapshot::build(
        txs.clone(),
        Vec::new(),
        &[],
        KeyIndex::default(),
        chunks(&txs),
        FeeRate::default(),
    );
    for i in 0..txs.len() {
        let seed = TxIndex::from(i);
        let ours = snapshot.cpfp_info_at(seed, txs[i].sigops);
        let mut graph = Cluster::default();
        let members = cluster::load(&txs, &cluster::walk(&txs, seed), &mut graph);
        let local = members.iter().position(|&m| m == seed).unwrap() as u32;
        let (cluster, rate) = graph.project(CpfpClusterTxIndex::from(local), |l| {
            let t = &txs[members[l].as_usize()];
            (t.txid, t.weight)
        });
        assert_eq!(
            to_value(&ours.cluster).unwrap(),
            to_value(&cluster).unwrap()
        );
        assert_eq!(ours.effective_fee_per_vsize, rate);
    }
}

/// Every component's chunks, as the snapshot builds them.
fn chunks(txs: &[SnapTx]) -> Chunks {
    let mut chunks = Chunks::default();
    let mut visited = vec![false; txs.len()];
    let mut graph = Cluster::default();
    for seed in 0..txs.len() {
        if visited[seed] {
            continue;
        }
        let component = cluster::walk(txs, TxIndex::from(seed));
        for m in &component {
            visited[m.as_usize()] = true;
        }
        let order = cluster::load(txs, &component, &mut graph);
        graph.linearize(|local, rate| {
            chunks.push_chunk(local.iter().map(|&l| order[u32::from(l) as usize]), rate);
        });
        chunks.end_component();
    }
    chunks
}

/// What block 0 leaves of a component packs as the clusters it now forms (a miss of one piece
/// defers that piece alone), and equal feerates go as Core 31's chunk index orders them: the
/// chunk whose greatest txid is lower first.
#[test]
fn projected_blocks_split_leftovers_and_order_ties() {
    let tx = |seed: u8, fee: u64, weight: u64, parents: &[usize]| SnapTx {
        txid: txid(seed),
        fee: Sats::from(fee),
        modified: fee as i64,
        vsize: VSize::from(Weight::from(weight)),
        weight: Weight::from(weight),
        size: 0,
        sigops: SigOps::ZERO,
        chunk_rate: FeeRate::default(),
        parents: parents.iter().map(|&p| TxIndex::from(p)).collect(),
        children: SmallVec::new(),
    };
    let with_children = |mut txs: Vec<SnapTx>| {
        for i in 0..txs.len() {
            for p in txs[i].parents.clone() {
                txs[p.as_usize()].children.push(TxIndex::from(i));
            }
        }
        txs
    };
    let packed = |txs: &[SnapTx], excluded: &[bool]| {
        partition(txs, &chunks(txs), excluded, 7)
            .blocks
            .iter()
            .map(|block| block.iter().map(|&(m, _)| m.as_usize()).collect::<Vec<_>>())
            .collect::<Vec<_>>()
    };
    // Block 0 took A (0), the parent of B (1) and C (2). D (3) fills most of the next block, so B
    // misses it; C, its own cluster now, still fits.
    let txs = with_children(vec![
        tx(1, 1_000, 400, &[]),
        tx(2, 2_000_000, 1_000_000, &[0]),
        tx(3, 4_000, 4_000, &[0]),
        tx(4, 9_000_000, 3_000_000, &[]),
    ]);
    assert_eq!(
        packed(&txs, &[true, false, false, false]),
        [vec![3, 2], vec![1]]
    );
    // Equal feerates: the lower greatest txid first, whatever the push order.
    let txs = with_children(vec![tx(9, 1_000, 400, &[]), tx(5, 1_000, 400, &[])]);
    assert_eq!(packed(&txs, &[false, false]), [vec![1, 0]]);
}

/// Core's budgets are strict: a block holds transactions up to 3,991,999 WU and 79,599 sigops; the
/// chunk reaching either limit waits for the next block.
#[test]
fn projected_blocks_stop_short_of_each_budget() {
    let tx = |seed: u8, fee: u64, weight: u64, sigops: u64| SnapTx {
        txid: txid(seed),
        fee: Sats::from(fee),
        modified: fee as i64,
        vsize: SigOps::new(sigops).adjust_vsize(VSize::from(Weight::from(weight))),
        weight: Weight::from(weight),
        size: 0,
        sigops: SigOps::new(sigops),
        chunk_rate: FeeRate::default(),
        parents: SmallVec::new(),
        children: SmallVec::new(),
    };
    let packed = |txs: &[SnapTx]| {
        let excluded = vec![false; txs.len()];
        partition(txs, &chunks(txs), &excluded, 7)
            .blocks
            .iter()
            .map(|block| block.iter().map(|&(m, _)| m.as_usize()).collect::<Vec<_>>())
            .collect::<Vec<_>>()
    };
    // Weight: 3,000,000 + 991,999 fill the block to one WU short; one more WU waits.
    let by_weight = [
        tx(1, 3_000_000, 3_000_000, 0),
        tx(2, 900_000, 991_999, 0),
        tx(3, 0, 1, 0),
    ];
    assert_eq!(packed(&by_weight), [vec![0, 1], vec![2]]);
    // Sigops: 79,599 fill the block to one short; one more waits.
    let by_sigops = [tx(1, 10_000_000, 1_600_000, 79_599), tx(2, 1, 400, 1)];
    assert_eq!(packed(&by_sigops), [vec![0], vec![1]]);
}

/// Served blocks through publication: block 0 and the projection's, but the dump, past the
/// projection; an unchanged block keeps its hash; a diff
/// against a block's past copy keeps runs, sends a body changed under its txid anew (not as
/// removed), knows no other block's hash, and forgets copies history evicted.
#[test]
fn served_blocks_publish_and_diff() {
    let tip = BlockHash::default();
    let (a, b, c, d, e) = (body(1), body(2), body(3), body(4), body(5));
    let mut state = ReadOnlyState::default();
    state.publish_projection(snapshot(&[&[&a], &[&b, &c, &d], &[&e]], 1), tip);
    let (block0, since) = (served(&state, 0).hash(), served(&state, 1).hash());
    assert!(state.mempool_block_source(2).unwrap().is_none(), "the dump");
    assert!(
        state.mempool_block_source(3).unwrap().is_none(),
        "past the projection"
    );

    let captured = served(&state, 1);
    // c's body changes under its txid (a prevout filled), d leaves block 1, e joins it.
    let c2 = Arc::new(Transaction {
        total_sigop_cost: SigOps::new(1),
        ..(*c).clone()
    });
    state.publish_projection(snapshot(&[&[&a], &[&b, &c2, &e]], 1), tip);
    assert_eq!(
        served(&state, 0).hash(),
        block0,
        "an unchanged block keeps its hash"
    );
    let built = to_value(captured.build()).unwrap();
    assert_eq!(
        built["hash"],
        to_value(since).unwrap(),
        "a capture outlives newer publications"
    );
    let bodies: Vec<&Value> = built["transactions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tx| &tx["sigops"])
        .collect();
    let txids: Vec<&Value> = built["transactions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tx| &tx["txid"])
        .collect();
    let id = |tx: &Arc<Transaction>| to_value(tx.txid).unwrap();
    assert_eq!(
        txids,
        [&id(&b), &id(&c), &id(&d)],
        "its own bodies, in its order"
    );
    assert_eq!(bodies[1], 0, "c's body as it was then");
    assert_ne!(served(&state, 1).hash(), since);
    let diff = state
        .resolve_mempool_block_diff(served(&state, 1), since)
        .expect("the past copy in history")
        .build();
    let diff = to_value(&diff).unwrap();
    let order: Vec<Value> = diff["order"].as_array().unwrap().clone();
    assert_eq!(order[0], serde_json::json!([0, 1]), "b kept");
    let txid = |tx: &Arc<Transaction>| to_value(tx.txid).unwrap();
    assert_eq!(
        [&order[1]["txid"], &order[2]["txid"]],
        [&txid(&c2), &txid(&e)]
    );
    assert_eq!(order[1]["sigops"], 1, "c's new body");
    assert_eq!(
        diff["removed"],
        serde_json::json!([txid(&d)]),
        "only d left"
    );
    assert!(
        state
            .resolve_mempool_block_diff(served(&state, 0), since)
            .is_none(),
        "block 1's hash is no copy of block 0"
    );

    // Past the history's bound (block 0 unchanged, each block 1 another pair of bodies).
    for first in 100..140 {
        for second in 140..170 {
            let block = [&body(first), &body(second)];
            state.publish_projection(snapshot(&[&[&a], &block], 1), tip);
        }
    }
    assert!(
        state
            .resolve_mempool_block_diff(served(&state, 1), since)
            .is_none(),
        "evicted"
    );
}

/// From an incomplete view, the blocks after block 0 withhold a replacement of a template-only
/// transaction (a conflict), a child of a listed transaction not loaded yet, and their
/// descendants; the rest stays.
#[test]
fn incomplete_views_withhold_what_cannot_follow_block_0() {
    let outpoint = |seed: u8, vout: u32| (txid(seed), Vout::from(vout));
    // 0: template-only, block 0, spends 90:0. 1: its replacement, spends 90:0 too. 2: 1's child.
    // 3: spends 91:0, listed but not loaded. 4: spends 92:0, confirmed. 5: 4's child.
    let bodies = [
        spending(10, &[outpoint(90, 0)]),
        spending(11, &[outpoint(90, 0)]),
        spending(12, &[outpoint(11, 0)]),
        spending(13, &[outpoint(91, 0)]),
        spending(14, &[outpoint(92, 0)]),
        spending(15, &[outpoint(14, 0)]),
    ];
    let mut txs: Vec<SnapTx> = bodies
        .iter()
        .map(|tx| SnapTx {
            txid: tx.txid,
            fee: tx.fee,
            modified: u64::from(tx.fee) as i64,
            vsize: VSize::from(tx.weight),
            weight: tx.weight,
            size: tx.total_size as u64,
            sigops: SigOps::ZERO,
            chunk_rate: FeeRate::from((tx.fee, tx.weight)),
            parents: SmallVec::new(),
            children: SmallVec::new(),
        })
        .collect();
    for (parent, child) in [(1, 2), (4, 5)] {
        txs[child].parents.push(TxIndex::from(parent));
        txs[parent].children.push(TxIndex::from(child));
    }
    let template_only: RapidHashSet<Txid> = [txid(10)].into_iter().collect();
    let unloaded: RapidHashSet<Txid> = [txid(91)].into_iter().collect();
    let mut excluded = vec![true, false, false, false, false, false];
    let bodies: Vec<&Transaction> = bodies.iter().collect();
    withhold(&bodies, &txs, &template_only, &unloaded, &mut excluded);
    assert_eq!(excluded, [true, true, true, true, false, false]);
}

/// A body pairs with its entry only as the variant the entry describes: another witness of the
/// same txid (swapped in between the two calls, another weight) stays unloaded.
#[test]
fn entries_pair_only_with_their_own_witness_variant() {
    let variant = |witness: &[u8]| bitcoin::Transaction {
        version: bitcoin::transaction::Version::TWO,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![bitcoin::TxIn {
            previous_output: bitcoin::OutPoint::new(bitcoin::Txid::from_byte_array([9; 32]), 0),
            script_sig: ScriptBuf::new(),
            sequence: bitcoin::Sequence::MAX,
            witness: Witness::from_slice(&[witness]),
        }],
        output: Vec::new(),
    };
    let (described, fetched) = (variant(&[1; 20]), variant(&[2; 99]));
    let txid = Txid::from(fetched.compute_txid());
    assert_eq!(txid, Txid::from(described.compute_txid()));
    let entry = |of: &bitcoin::Transaction| MempoolEntryInfo {
        txid,
        wtxid: of.compute_wtxid(),
        vsize: VSize::from(Weight::from(of.weight())),
        weight: Weight::from(of.weight()),
        fee: Sats::from(1_000u64),
        first_seen: Timestamp::now(),
    };
    let state = State::default();
    let added = |entry: MempoolEntryInfo| {
        let bodies = [(txid, fetched.clone())].into_iter().collect();
        preparer::prepare(
            &[txid],
            &RapidHashSet::default(),
            vec![entry],
            bodies,
            &Confirmed::default(),
            &state,
        )
        .added
        .len()
    };
    assert_eq!(added(entry(&described)), 0, "another variant's entry");
    assert_eq!(added(entry(&fetched)), 1, "its own entry");
}

/// Through ingestion and `Rebuilder::tick`: the blocks after block 0 withhold a template-only
/// transaction's replacement and an unloaded parent's child; the same view, gaps included, reuses
/// the snapshot; either gap changing alone rebuilds it, and what it withholds follows.
#[test]
fn rebuilds_follow_the_view_gaps() {
    let raw = |spends: (bitcoin::Txid, u32), sequence: u32| bitcoin::Transaction {
        version: bitcoin::transaction::Version::TWO,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![bitcoin::TxIn {
            previous_output: bitcoin::OutPoint::new(spends.0, spends.1),
            script_sig: ScriptBuf::new(),
            sequence: bitcoin::Sequence(sequence),
            witness: Witness::new(),
        }],
        output: vec![bitcoin::TxOut {
            value: bitcoin::Amount::from_sat(1_000),
            script_pubkey: ScriptBuf::new(),
        }],
    };
    let confirmed = |seed: u8| bitcoin::Txid::from_byte_array([seed; 32]);
    let unloaded_parent = confirmed(95);
    // p: unrelated; t: template-only; r: t's replacement (same outpoint); c: child of a listed
    // transaction not loaded yet.
    let p = raw((confirmed(90), 0), 0);
    let t = raw((confirmed(91), 0), 1);
    let r = raw((confirmed(91), 0), 2);
    let c = raw((unloaded_parent, 0), 0);
    let id = |tx: &bitcoin::Transaction| Txid::from(tx.compute_txid());
    let mut state = State::default();
    let template_only: RapidHashSet<Txid> = [id(&t)].into_iter().collect();
    let entries = [&p, &t, &r, &c]
        .iter()
        .map(|tx| MempoolEntryInfo {
            txid: id(tx),
            wtxid: tx.compute_wtxid(),
            vsize: VSize::from(Weight::from(tx.weight())),
            weight: Weight::from(tx.weight()),
            fee: Sats::from(1_000u64),
            first_seen: Timestamp::now(),
        })
        .collect();
    let bodies = [&p, &t, &r, &c]
        .iter()
        .map(|&tx| (id(tx), tx.clone()))
        .collect();
    let live = [id(&p), id(&r), id(&c), Txid::from(unloaded_parent), id(&t)];
    let diff = preparer::prepare(
        &live,
        &template_only,
        entries,
        bodies,
        &Confirmed::default(),
        &state,
    );
    applier::apply(
        &mut state,
        &Snapshot::default(),
        diff,
        &Confirmed::default(),
    );
    state.template_only = template_only;

    let mut rebuilder = Rebuilder::default();
    let mut unloaded: RapidHashSet<Txid> = [Txid::from(unloaded_parent)].into_iter().collect();
    let mut tick = |state: &State, unloaded: &RapidHashSet<Txid>| {
        rebuilder.tick(state, &[id(&t)], FeeRate::default(), &[], unloaded);
        let snapshot = rebuilder.snapshot().clone();
        let later: RapidHashSet<Txid> = snapshot.served_blocks()[1..]
            .iter()
            .flat_map(|block| block.bodies.iter().map(|tx| tx.txid))
            .collect();
        (snapshot, later)
    };
    let set = |txids: &[Txid]| txids.iter().copied().collect::<RapidHashSet<Txid>>();
    let (first, later) = tick(&state, &unloaded);
    assert_eq!(later, set(&[id(&p)]), "r and c withheld");
    let (again, _) = tick(&state, &unloaded);
    assert!(
        Arc::ptr_eq(&first, &again),
        "the same view reuses its snapshot"
    );

    unloaded.clear();
    let (loaded, later) = tick(&state, &unloaded);
    assert!(!Arc::ptr_eq(&again, &loaded), "the unloaded gap changed");
    assert_eq!(
        later,
        set(&[id(&p), id(&c)]),
        "c follows once its parent isn't missing"
    );

    state.template_only.clear();
    let (listed, later) = tick(&state, &unloaded);
    assert!(
        !Arc::ptr_eq(&loaded, &listed),
        "the template-only gap changed"
    );
    assert_eq!(later, set(&[id(&p), id(&r), id(&c)]));
}

fn served(state: &ReadOnlyState, index: usize) -> MempoolBlockSource {
    state.mempool_block_source(index).unwrap().expect("served")
}

/// A snapshot projecting `blocks` (block 0 first), block 0 and the `later` blocks after it served.
fn snapshot(blocks: &[&[&Arc<Transaction>]], later: usize) -> Arc<Snapshot> {
    let mut txs: Vec<SnapTx> = Vec::new();
    let indexed: Vec<Vec<(TxIndex, FeeRate)>> = blocks
        .iter()
        .map(|block| {
            block
                .iter()
                .map(|tx| {
                    let rate = FeeRate::from((tx.fee, tx.weight));
                    txs.push(SnapTx {
                        txid: tx.txid,
                        fee: tx.fee,
                        modified: u64::from(tx.fee) as i64,
                        vsize: VSize::from(tx.weight),
                        weight: tx.weight,
                        size: tx.total_size as u64,
                        sigops: tx.total_sigop_cost,
                        chunk_rate: rate,
                        parents: SmallVec::new(),
                        children: SmallVec::new(),
                    });
                    (TxIndex::from(txs.len() - 1), rate)
                })
                .collect()
        })
        .collect();
    let block0 = indexed[0].iter().map(|&(index, _)| index).collect();
    let mut snapshot = Snapshot::build(
        txs,
        block0,
        &indexed[1..],
        KeyIndex::default(),
        Chunks::default(),
        FeeRate::default(),
    );
    let bodies = blocks[..=later]
        .iter()
        .map(|block| {
            block
                .iter()
                .map(|&tx| (tx.clone(), TxRecord::digest_of(tx)))
                .unzip()
        })
        .collect();
    snapshot.set_served(bodies, 0, false);
    Arc::new(snapshot)
}

fn txid(seed: u8) -> Txid {
    Txid::from(bitcoin::Txid::from_byte_array([seed; 32]))
}

/// [`body`] spending `outpoints`.
fn spending(seed: u8, outpoints: &[(Txid, Vout)]) -> Transaction {
    let mut tx = (*body(seed)).clone();
    tx.input = outpoints
        .iter()
        .map(|&(txid, vout)| TxIn {
            txid,
            vout,
            prevout: None,
            script_sig: ScriptBuf::new(),
            witness: Witness::new(),
            is_coinbase: false,
            sequence: u32::MAX,
        })
        .collect();
    tx
}

/// A bare unconfirmed transaction, its txid and fee from `seed`.
fn body(seed: u8) -> Arc<Transaction> {
    Arc::new(Transaction {
        index: None,
        txid: txid(seed),
        version: bitcoin::transaction::Version::TWO.into(),
        lock_time: bitcoin::absolute::LockTime::ZERO.into(),
        input: Vec::new(),
        output: Vec::new(),
        total_size: 100,
        weight: Weight::from(400u64),
        total_sigop_cost: SigOps::ZERO,
        fee: Sats::from(1_000 * u64::from(seed)),
        status: TxStatus::UNCONFIRMED,
    })
}
