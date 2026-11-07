use std::sync::Arc;

use bitcoin::hashes::Hash;
use bitcoin::{ScriptBuf, Witness};
use brk_cpfp::Cluster;
use brk_types::{
    BlockHash, CpfpClusterTxIndex, FeeRate, MempoolEntryInfo, Sats, SigOps, Timestamp, Transaction,
    TxIn, TxOut, TxStatus, Txid, VSize, Vout, Weight,
};
use rapidhash::{RapidHashMap, RapidHashSet};
use serde_json::{Value, to_value};
use smallvec::SmallVec;

use crate::{
    MempoolBlockSource, RbfNode, ReadOnlyState, State,
    snapshot::Rebuilder,
    snapshot::{
        SnapTx, Snapshot, TxIndex,
        builder::KeyIndex,
        cluster,
        partition::{Chunks, partition},
        rebuilder::withhold,
    },
    steps::{Confirmed, applier, preparer::{self, TxAddition}, prevouts},
    state::TxEntry,
    stores::{Linker, TxGraveyard, TxStore, tx_store::TxRecord},
    steps::TxRemoval,
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
        false,
        &[],
        KeyIndex::default(),
        chunks(&txs),
        FeeRate::default(),
        None,
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
/// defers that piece alone), equal feerates go as Core 31's chunk index orders them (the chunk
/// whose greatest txid is lower first), and what Core's block builder would never take waits in
/// the catch-all.
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
    // Below Core's block floor (an operator's negative delta): never packed, in the catch-all.
    let mut txs = with_children(vec![tx(6, 1_000, 400, &[]), tx(7, 1_000, 400, &[])]);
    txs[1].modified = -500;
    let projection = partition(&txs, &chunks(&txs), &[false, false], 7);
    assert!(projection.overflow);
    let blocks: Vec<Vec<usize>> = projection
        .blocks
        .iter()
        .map(|block| block.iter().map(|&(m, _)| m.as_usize()).collect())
        .collect();
    assert_eq!(blocks, [vec![0], vec![1]]);
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
    let built: Value = serde_json::from_slice(&captured.json()).unwrap();
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
    // 0: a replacement of 5, spends 90:0. 1: 0's child. 2: spends 91:0, listed but not loaded.
    // 3: spends 92:0, confirmed. 4: 3's child. 5: template-only (after the pool's), block 0,
    // spends 90:0 too.
    let bodies = [
        spending(11, &[outpoint(90, 0)]),
        spending(12, &[outpoint(11, 0)]),
        spending(13, &[outpoint(91, 0)]),
        spending(14, &[outpoint(92, 0)]),
        spending(15, &[outpoint(14, 0)]),
        spending(10, &[outpoint(90, 0)]),
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
    for (parent, child) in [(0, 1), (3, 4)] {
        txs[child].parents.push(TxIndex::from(parent));
        txs[parent].children.push(TxIndex::from(child));
    }
    let unloaded: RapidHashSet<Txid> = [txid(91)].into_iter().collect();
    let mut excluded = vec![false, false, false, false, false, true];
    let bodies: Vec<&Transaction> = bodies.iter().collect();
    withhold(&bodies, &txs, 5, &unloaded, &mut excluded);
    assert_eq!(excluded, [true, true, true, false, false, true]);
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
    let info = |tx: &bitcoin::Transaction| MempoolEntryInfo {
        txid: id(tx),
        wtxid: tx.compute_wtxid(),
        vsize: VSize::from(Weight::from(tx.weight())),
        weight: Weight::from(tx.weight()),
        fee: Sats::from(1_000u64),
        first_seen: Timestamp::now(),
    };
    let entries = [&p, &r, &c].iter().map(|tx| info(tx)).collect();
    let bodies = [&p, &r, &c]
        .iter()
        .map(|&tx| (id(tx), tx.clone()))
        .collect();
    let live = [id(&p), id(&r), id(&c), Txid::from(unloaded_parent)];
    let diff = preparer::prepare(&live, entries, bodies, &Confirmed::default(), &state);
    applier::apply(
        &mut state,
        &Snapshot::default(),
        diff,
        &Confirmed::default(),
    );
    let TxAddition { tx, entry } = TxAddition::new(&info(&t), t.clone(), &state.txs, &state.graveyard);
    state.template_only = vec![TxRecord::new(Arc::new(tx), entry)];

    let mut rebuilder = Rebuilder::default();
    let mut unloaded: RapidHashSet<Txid> = [Txid::from(unloaded_parent)].into_iter().collect();
    let mut tick = |state: &State, unloaded: &RapidHashSet<Txid>| {
        rebuilder.tick(state, Some(&[id(&t)]), FeeRate::default(), &[], unloaded, unloaded.is_empty());
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

    // A listed transaction stuck unloaded withholds what follows it, but holds no tiers back,
    // even when nothing else changed since it was only missing.
    let stuck = set(&[Txid::from(unloaded_parent)]);
    let mut rebuilder = Rebuilder::default();
    for ready in [false, true] {
        rebuilder.tick(&state, None, FeeRate::default(), &[], &stuck, ready);
        assert_eq!(rebuilder.snapshot().ensure_fees().is_ok(), ready);
    }
}

/// An address's pending view, through ingestion, prevout fills and a publication: a transaction
/// folds in as far as it is known (h's output at once, its input once resolved, and its child hc
/// whole, h's output being pooled); behind the
/// reader, the pool goes without what the newer blocks mined (m), conflicted (c) or what descends
/// from a conflicted one (d), while a mined one's child (k) and the unrelated (u) stay pending.
#[test]
fn pending_views_settle_what_newer_blocks_mined_or_conflicted() {
    let script = ScriptBuf::new_p2wpkh(&bitcoin::WPubkeyHash::from_byte_array([7; 20]));
    let raw = |spends: bitcoin::OutPoint, sats: u64| bitcoin::Transaction {
        version: bitcoin::transaction::Version::TWO,
        lock_time: bitcoin::absolute::LockTime::ZERO,
        input: vec![bitcoin::TxIn {
            previous_output: spends,
            script_sig: ScriptBuf::new(),
            sequence: bitcoin::Sequence::MAX,
            witness: Witness::new(),
        }],
        output: vec![bitcoin::TxOut {
            value: bitcoin::Amount::from_sat(sats),
            script_pubkey: script.clone(),
        }],
    };
    let confirmed = |seed: u8| bitcoin::OutPoint::new(bitcoin::Txid::from_byte_array([seed; 32]), 0);
    let m = raw(confirmed(80), 1_000);
    let k = raw(bitcoin::OutPoint::new(m.compute_txid(), 0), 600);
    let c = raw(confirmed(81), 700);
    let d = raw(bitcoin::OutPoint::new(c.compute_txid(), 0), 300);
    let u = raw(confirmed(82), 200);
    // Its prevout resolves late; its child's (h's output) at once. Both pay another address.
    let other_script = ScriptBuf::new_p2wpkh(&bitcoin::WPubkeyHash::from_byte_array([8; 20]));
    let mut h = raw(confirmed(83), 50);
    h.output[0].script_pubkey = other_script.clone();
    let mut hc = raw(bitcoin::OutPoint::new(h.compute_txid(), 0), 25);
    hc.output[0].script_pubkey = other_script.clone();
    let txs = [&m, &k, &c, &d, &u, &h, &hc];
    let id = |tx: &bitcoin::Transaction| Txid::from(tx.compute_txid());
    let mut state = State::default();
    let entries = txs
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
    let bodies = txs.iter().map(|&tx| (id(tx), tx.clone())).collect();
    let live: Vec<Txid> = txs.iter().map(|tx| id(tx)).collect();
    let diff = preparer::prepare(&live, entries, bodies, &Confirmed::default(), &state);
    applier::apply(&mut state, &Snapshot::default(), diff, &Confirmed::default());
    let funding = TxOut::from((script.clone(), Sats::from(5_000u64)));
    let addr = funding.addr_bytes().unwrap();
    prevouts::fill(&mut state, |holes: &[(Txid, Vout)]| {
        holes
            .iter()
            .filter(|(txid, _)| *txid != Txid::from(confirmed(83).txid))
            .map(|&hole| (hole, funding.clone()))
            .collect::<RapidHashMap<_, _>>()
    }, &RapidHashSet::default());
    let mut rebuilder = Rebuilder::default();
    let tip = BlockHash::default();
    let mut publish = |state: &State, previous: &ReadOnlyState| {
        rebuilder.tick(state, None, FeeRate::default(), &[], &RapidHashSet::default(), true);
        previous
            .updated(state, tip, rebuilder.snapshot().clone(), true)
            .expect("published")
    };
    let read = publish(&state, &ReadOnlyState::default());

    let at = read.view_at(&tip).unwrap();
    let stats = read.addr_stats(&addr, &at).unwrap();
    // h's input isn't known yet; each confirmed input spends one of the address's outputs.
    assert_eq!(stats.tx_count, 5);
    assert_eq!(u64::from(stats.funded_txo_sum), 2_800);
    assert_eq!(u64::from(stats.spent_txo_sum), 3 * 5_000 + 1_000 + 700);
    assert!(!read.addr_has_tx(&addr, &id(&h), &at).unwrap(), "what it spends isn't known yet");
    let other = TxOut::from((other_script.clone(), Sats::ZERO)).addr_bytes().unwrap();
    let stats = read.addr_stats(&other, &at).unwrap();
    assert_eq!(stats.tx_count, 2, "h's output, and hc spending it");
    assert_eq!(u64::from(stats.funded_txo_sum), 50 + 25);
    assert_eq!(u64::from(stats.spent_txo_sum), 50);

    // Past the pool's tip: a block mined m and spent c's input in another transaction.
    let conflict = id(&raw(confirmed(81), 1));
    let to_spend = |outpoint: bitcoin::OutPoint| (Txid::from(outpoint.txid), Vout::from(0usize));
    let reader = BlockHash::from(bitcoin::BlockHash::from_byte_array([2; 32]));
    let spends = || -> brk_error::Result<_> {
        Ok([(to_spend(confirmed(80)), id(&m)), (to_spend(confirmed(81)), conflict)])
    };
    let behind = read.view_behind(reader, spends).unwrap();
    let again = read
        .view_behind(reader, || -> brk_error::Result<[_; 0]> {
            panic!("read once for the moment")
        })
        .unwrap();
    assert!(Arc::ptr_eq(&again, &behind), "one view for every read of that moment");
    let stats = read.addr_stats(&addr, &behind).unwrap();
    assert_eq!(stats.tx_count, 2, "k and u");
    assert_eq!(u64::from(stats.funded_txo_sum), 800);
    assert_eq!(u64::from(stats.spent_txo_sum), 1_000 + 5_000, "m's output, u's input");
    let page: RapidHashSet<Txid> = read
        .addr_txs(&addr, None, 10, &behind)
        .unwrap()
        .iter()
        .map(|published| published.tx.txid)
        .collect();
    assert_eq!(page, [id(&k), id(&u)].into_iter().collect());
    for (tx, pending) in [(&m, false), (&k, true), (&c, false), (&d, false), (&u, true)] {
        assert_eq!(read.is_pending(&id(tx), &behind).unwrap(), pending);
    }
    let utxos = read.addr_pending_utxos(&addr, &behind).unwrap();
    assert_eq!(utxos.created.len(), 2, "k's and u's outputs");
    let mut spends = utxos.spends();
    assert!(spends.spent(&id(&m), Vout::from(0usize)));
    assert_ne!(spends.token(), utxos.token, "the spenders met are part of the token");
    assert!(
        read.view_at(&BlockHash::from(bitcoin::BlockHash::from_byte_array([1; 32])))
            .is_err(),
        "another tip"
    );

    prevouts::fill(&mut state, |holes: &[(Txid, Vout)]| {
        holes.iter().map(|&hole| (hole, funding.clone())).collect::<RapidHashMap<_, _>>()
    }, &RapidHashSet::default());
    let read = publish(&state, &read);
    let at = read.view_at(&tip).unwrap();
    let stats = read.addr_stats(&addr, &at).unwrap();
    assert_eq!(stats.tx_count, 6, "h spends one of its outputs");
    assert_eq!(u64::from(stats.spent_txo_sum), 4 * 5_000 + 1_000 + 700);
    assert!(read.addr_has_tx(&addr, &id(&h), &at).unwrap());
    // Pages: newest first, a cursor continuing after itself, ending past the oldest.
    let page = |after: Option<&Txid>, limit| -> Vec<Txid> {
        read.addr_txs(&addr, after, limit, &at)
            .unwrap()
            .iter()
            .map(|published| published.tx.txid)
            .collect()
    };
    let all = page(None, 10);
    assert_eq!(all.len(), 6);
    let (first, second) = (page(None, 2), page(Some(&all[1]), 10));
    assert_eq!(first, all[..2]);
    assert_eq!(second, all[2..]);
    assert!(page(Some(&all[5]), 10).is_empty());
}

/// After a reorg: what a dropped block mined reads vanished, or replaced by a conflict the new
/// blocks confirmed; what the new blocks mined again, or no dropped block held, stays mined.
#[test]
fn reorged_out_tombstones_read_vanished_or_replaced() {
    let (mut graveyard, mut linker) = (TxGraveyard::default(), Linker::default());
    let outpoint = |seed: u8| (txid(80 + seed), Vout::from(0usize));
    let bury = |graveyard: &mut TxGraveyard, linker: &mut Linker, seed: u8| {
        let tx = Arc::new(spending(seed, &[outpoint(seed)]));
        let info = MempoolEntryInfo {
            txid: tx.txid,
            wtxid: bitcoin::Wtxid::from_byte_array([seed; 32]),
            vsize: VSize::from(tx.weight),
            weight: tx.weight,
            fee: tx.fee,
            first_seen: Timestamp::now(),
        };
        let record = TxRecord::new(tx, TxEntry::new(&info, 100, false));
        graveyard.bury(record, FeeRate::default(), TxRemoval::Mined, linker);
    };
    // 1: dropped, nothing in its place; 2: dropped, a conflict confirmed; 3: dropped, mined again;
    // 4: its block stayed.
    for seed in 1..=4 {
        bury(&mut graveyard, &mut linker, seed);
    }
    graveyard.confirm(&Confirmed::of(
        [txid(3), txid(9)],
        [(outpoint(2), txid(9)), (outpoint(3), txid(3))],
        [1, 2, 3].map(txid),
    ), &mut linker);
    let removal = |seed| graveyard.get(&txid(seed)).unwrap().removal;
    assert!(matches!(removal(1), TxRemoval::Vanished));
    assert!(matches!(removal(2), TxRemoval::Replaced { by } if by == txid(9)));
    assert!(matches!(removal(3), TxRemoval::Mined));
    assert!(matches!(removal(4), TxRemoval::Mined));
}

/// A replacer the fetch cap held back links what vanished before it arrived, if it was listed by
/// then; one first listed later (spending an outpoint the vanished one freed) replaced nothing;
/// after a complete cycle, nothing is left waiting for one.
#[test]
fn late_replacers_relink_what_vanished_before_them() {
    let (mut graveyard, mut linker) = (TxGraveyard::default(), Linker::default());
    let outpoint = |seed: u8| (txid(seed), Vout::from(0usize));
    let bury = |graveyard: &mut TxGraveyard, linker: &mut Linker, seed: u8, spends: &[u8]| {
        let outpoints: Vec<(Txid, Vout)> = spends.iter().map(|&s| outpoint(s)).collect();
        let tx = Arc::new(spending(seed, &outpoints));
        let info = MempoolEntryInfo {
            txid: tx.txid,
            wtxid: bitcoin::Wtxid::from_byte_array([seed; 32]),
            vsize: VSize::from(tx.weight),
            weight: tx.weight,
            fee: tx.fee,
            first_seen: Timestamp::now(),
        };
        let record = TxRecord::new(tx, TxEntry::new(&info, 100, false));
        graveyard.bury(record, FeeRate::default(), TxRemoval::Vanished, linker);
    };
    let unloaded = |seeds: &[u8]| seeds.iter().map(|&s| txid(s)).collect::<RapidHashSet<Txid>>();
    let removal = |graveyard: &TxGraveyard, seed| graveyard.get(&txid(seed)).unwrap().removal;

    // 1 spent 90 and 91 and vanished while 2, its replacer (spending 91), was listed unloaded.
    bury(&mut graveyard, &mut linker, 1, &[90, 91]);
    graveyard.index_vanished(&mut linker);
    graveyard.end_cycle(&unloaded(&[2]), &mut linker);
    // 3 is first listed after: it spends 90, which 1's leaving freed.
    graveyard.end_cycle(&unloaded(&[2, 3]), &mut linker);
    graveyard.relink(&spending(3, &[outpoint(90)]), &mut linker);
    assert!(matches!(removal(&graveyard, 1), TxRemoval::Vanished));
    graveyard.relink(&spending(2, &[outpoint(91)]), &mut linker);
    assert!(matches!(removal(&graveyard, 1), TxRemoval::Replaced { by } if by == txid(2)));
    assert_eq!(graveyard.predecessors_of(&txid(2)).count(), 1);

    bury(&mut graveyard, &mut linker, 4, &[92]);
    graveyard.index_vanished(&mut linker);
    graveyard.end_cycle(&unloaded(&[]), &mut linker);
    graveyard.relink(&spending(5, &[outpoint(92)]), &mut linker);
    assert!(matches!(removal(&graveyard, 4), TxRemoval::Vanished));
}

/// A confirmed conflict replaced what vanished since the blocks were last all told, or what a late
/// replacer listed by then displaced; an outpoint respent after its transaction left replaced
/// nothing.
#[test]
fn confirmed_conflicts_replace_only_what_they_displaced() {
    let (mut graveyard, mut linker) = (TxGraveyard::default(), Linker::default());
    let outpoint = |seed: u8| (txid(seed), Vout::from(0usize));
    let bury = |graveyard: &mut TxGraveyard, linker: &mut Linker, seed: u8, spends: &[u8]| {
        let outpoints: Vec<(Txid, Vout)> = spends.iter().map(|&s| outpoint(s)).collect();
        let tx = Arc::new(spending(seed, &outpoints));
        let info = MempoolEntryInfo {
            txid: tx.txid,
            wtxid: bitcoin::Wtxid::from_byte_array([seed; 32]),
            vsize: VSize::from(tx.weight),
            weight: tx.weight,
            fee: tx.fee,
            first_seen: Timestamp::now(),
        };
        let record = TxRecord::new(tx, TxEntry::new(&info, 100, false));
        graveyard.bury(record, FeeRate::default(), TxRemoval::Vanished, linker);
    };
    // 1 left (evicted, say), and 3 (spending 92 and 93) while 8, spending 93, was listed
    // unloaded; the cycle's blocks were all told.
    bury(&mut graveyard, &mut linker, 1, &[90]);
    bury(&mut graveyard, &mut linker, 3, &[92, 93]);
    graveyard.index_vanished(&mut linker);
    graveyard.end_cycle(&[txid(8)].into_iter().collect(), &mut linker);
    graveyard.set_told(BlockHash::from(bitcoin::BlockHash::from_byte_array([5; 32])), &mut linker);
    // 2 leaves as a block confirms 9 (spending 90 and 91), 7 (spending 92, which 3's leaving
    // freed) and 8.
    bury(&mut graveyard, &mut linker, 2, &[91]);
    let spend = |seed: u8, by: u8| (outpoint(seed), txid(by));
    graveyard.confirm(&Confirmed::of(
        [7, 8, 9].map(txid),
        [spend(90, 9), spend(91, 9), spend(92, 7), spend(93, 8)],
        [],
    ), &mut linker);
    let removal = |seed| graveyard.get(&txid(seed)).unwrap().removal;
    assert!(matches!(removal(1), TxRemoval::Vanished));
    assert!(matches!(removal(2), TxRemoval::Replaced { by } if by == txid(9)));
    assert!(matches!(removal(3), TxRemoval::Replaced { by } if by == txid(8)));
}

/// Retention: a lone tombstone goes an hour after its burial; a tree stays while its root is
/// pending (Core listing it is enough), and goes whole a day after the root left, or was last
/// seen listed; a tree a confirmed replacer the pool never saw roots goes a day after.
#[test]
fn tombstones_and_trees_leave_after_their_retention() {
    use crate::stores::Moment;
    use std::time::Duration;
    let (mut graveyard, mut linker) = (TxGraveyard::default(), Linker::default());
    let bury = |graveyard: &mut TxGraveyard, linker: &mut Linker, seed: u8, removal: TxRemoval| {
        let tx = Arc::new(spending(seed, &[(txid(100 + seed), Vout::from(0usize))]));
        let info = MempoolEntryInfo {
            txid: tx.txid,
            wtxid: bitcoin::Wtxid::from_byte_array([seed; 32]),
            vsize: VSize::from(tx.weight),
            weight: tx.weight,
            fee: tx.fee,
            first_seen: Timestamp::now(),
        };
        let record = TxRecord::new(tx, TxEntry::new(&info, 100, false));
        graveyard.bury(record, FeeRate::default(), removal, linker);
    };
    let replaced = |by: u8| TxRemoval::Replaced { by: txid(by) };
    // 1 lone; 2, replaced by 3, replaced by 4, pending; 5 replaced by 6, which a block confirmed
    // unseen; 7 replaced by 8, mined; 9 replaced by 10, vanished but listed again.
    bury(&mut graveyard, &mut linker, 1, TxRemoval::Vanished);
    bury(&mut graveyard, &mut linker, 2, replaced(3));
    bury(&mut graveyard, &mut linker, 3, replaced(4));
    bury(&mut graveyard, &mut linker, 5, replaced(6));
    bury(&mut graveyard, &mut linker, 7, replaced(8));
    bury(&mut graveyard, &mut linker, 8, TxRemoval::Mined);
    bury(&mut graveyard, &mut linker, 9, replaced(10));
    bury(&mut graveyard, &mut linker, 10, TxRemoval::Vanished);
    let start = Moment::now();
    let at = |hours: u64| start + Duration::from_secs(hours * 3_600 + 60);
    let holds = |graveyard: &TxGraveyard, seeds: &[u8]| -> Vec<bool> {
        seeds.iter().map(|&seed| graveyard.get(&txid(seed)).is_some()).collect()
    };
    let pending = |id: &Txid| *id == txid(4) || *id == txid(10);

    graveyard.evict_old(at(0), pending);
    assert!(graveyard.consistent());
    assert_eq!(holds(&graveyard, &[1, 2, 3, 5, 7, 8, 9, 10]), [true; 8]);
    graveyard.evict_old(at(1), pending);
    assert!(graveyard.consistent());
    assert_eq!(holds(&graveyard, &[1]), [false], "a lone one goes after an hour");
    graveyard.evict_old(at(23), pending);
    assert!(graveyard.consistent());
    assert_eq!(holds(&graveyard, &[2, 3, 5, 7, 8, 9, 10]), [true; 7], "trees stay a day");
    graveyard.evict_old(at(24), pending);
    assert!(graveyard.consistent());
    assert_eq!(holds(&graveyard, &[5, 7, 8]), [false; 3], "a day after their root left");
    assert_eq!(holds(&graveyard, &[2, 3, 9, 10]), [true; 4], "while their root is pending");
    // Neither root is listed any more.
    graveyard.evict_old(at(47), |_| false);
    assert!(graveyard.consistent());
    assert_eq!(holds(&graveyard, &[2, 3, 9, 10]), [true; 4], "a day after last seen pending");
    graveyard.evict_old(at(48), |_| false);
    assert!(graveyard.consistent());
    assert_eq!(holds(&graveyard, &[2, 3, 9, 10]), [false; 4], "whole trees go");
    assert_eq!(graveyard.predecessors_of(&txid(4)).count(), 0);
}

/// The graveyard through random burials (re-burials, chains, replacers buried and back), exhumes,
/// blocks, late replacers, cycle ends and evictions at random later moments: its invariants hold
/// after each. As in a cycle, a replacer with a tombstone arrives (exhumed) or is mined, and a
/// block mines what it spends with.
#[test]
fn graveyard_invariants_hold_through_random_histories() {
    use crate::stores::Moment;
    use std::time::Duration;
    let outpoint = |seed: u64| (txid(200 + (seed % 40) as u8), Vout::from(0usize));
    let body = |seed: u8, spends: u64| {
        let tx = Arc::new(spending(seed, &[outpoint(spends), outpoint(spends + 1)]));
        let info = MempoolEntryInfo {
            txid: tx.txid,
            wtxid: bitcoin::Wtxid::from_byte_array([seed; 32]),
            vsize: VSize::from(tx.weight),
            weight: tx.weight,
            fee: tx.fee,
            first_seen: Timestamp::now(),
        };
        TxRecord::new(tx, TxEntry::new(&info, 100, false))
    };
    for history in 0..16u64 {
        let mut state = 0xBB67_AE85_84CA_A73Bu64 ^ (history << 32 | history);
        let mut next = |bound: u64| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state % bound
        };
        let start = Moment::now();
        let (mut graveyard, mut linker) = (TxGraveyard::default(), Linker::default());
        let mut hours = 0u64;
        for _ in 0..4_000 {
            let seed = next(48) as u8;
            match next(12) {
                0..=4 => {
                    let removal = match next(3) {
                        0 => TxRemoval::Mined,
                        1 => TxRemoval::Vanished,
                        _ => TxRemoval::Replaced { by: txid(next(48) as u8) },
                    };
                    graveyard.bury(body(seed, next(40)), FeeRate::default(), removal, &mut linker);
                    if let TxRemoval::Replaced { by } = removal {
                        graveyard.exhume(&by);
                    }
                }
                5 => graveyard.exhume(&txid(seed)),
                6 => {
                    let spends = (0..next(6))
                        .map(|_| (outpoint(next(40)), txid(next(48) as u8)))
                        .collect::<Vec<_>>();
                    let mined = (0..next(4))
                        .map(|_| txid(next(48) as u8))
                        .chain(spends.iter().map(|(_, by)| *by))
                        .collect::<Vec<_>>();
                    let disconnected = (0..next(3)).map(|_| txid(next(48) as u8)).collect::<Vec<_>>();
                    graveyard.confirm(&Confirmed::of(mined, spends, disconnected), &mut linker);
                }
                7 => graveyard.set_told(
                    BlockHash::from(bitcoin::BlockHash::from_byte_array([next(4) as u8; 32])),
                    &mut linker,
                ),
                8 => {
                    graveyard.index_vanished(&mut linker);
                    let unloaded = (0..next(3)).map(|_| txid(next(48) as u8)).collect();
                    graveyard.end_cycle(&unloaded, &mut linker);
                }
                9 => {
                    graveyard.exhume(&txid(seed));
                    graveyard.relink(&spending(seed, &[outpoint(next(40))]), &mut linker);
                }
                _ => {
                    hours += next(30);
                    let listed = next(48) as u8;
                    graveyard.evict_old(start + Duration::from_secs(hours * 3_600), |id| {
                        *id == txid(listed)
                    });
                }
            }
            assert!(graveyard.consistent(), "graveyard invariants, history {history}");
        }
    }
}

/// A tree too wide for its budget still holds the version asked for, its siblings cut instead.
#[test]
fn wide_trees_keep_the_version_asked_for() {
    let mut state = State::default();
    let unseen = txid(200);
    let mut asked = None;
    for i in 0..4_200u32 {
        let mut bytes = [0xAB; 32];
        bytes[..4].copy_from_slice(&i.to_le_bytes());
        let mut tx = (*body(1)).clone();
        tx.txid = Txid::from(bitcoin::Txid::from_byte_array(bytes));
        let info = MempoolEntryInfo {
            txid: tx.txid,
            wtxid: bitcoin::Wtxid::from_byte_array(bytes),
            vsize: VSize::from(tx.weight),
            weight: tx.weight,
            fee: tx.fee,
            first_seen: Timestamp::now(),
        };
        asked = Some(tx.txid);
        let record = TxRecord::new(Arc::new(tx), TxEntry::new(&info, 100, true));
        state.graveyard.bury(
            record,
            FeeRate::default(),
            TxRemoval::Replaced { by: unseen },
            &mut state.linker,
        );
    }
    let mut rebuilder = Rebuilder::default();
    rebuilder.tick(&state, None, FeeRate::default(), &[], &RapidHashSet::default(), true);
    let tip = BlockHash::default();
    let read = ReadOnlyState::default()
        .updated(&state, tip, rebuilder.snapshot().clone(), true)
        .expect("published");
    let root = read.rbf_for_tx(&asked.unwrap(), &tip).unwrap().root.expect("a tree");
    assert!(root.truncated);
    assert!(root.replaces.len() < 4_200, "cut");
    assert!(
        root.replaces.iter().any(|node| Some(node.txid) == asked),
        "the one asked for is in it"
    );
}

/// The copy-on-write containers against plain models, through random inserts, swap-removes,
/// writes, pushes and pops: same contents and dense order, and a clone taken midway never sees a
/// later write.
#[test]
fn copy_on_write_containers_match_their_models() {
    use crate::stores::cow::{CowIndexMap, CowMap, CowQueue};
    use rustc_hash::FxBuildHasher;
    use std::collections::VecDeque;
    let mut state = 0x6A09_E667_F3BC_C908u64;
    let mut next = |bound: u64| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state % bound
    };
    let mut map: CowIndexMap<u64, u64, FxBuildHasher> = CowIndexMap::default();
    let mut model: Vec<(u64, u64)> = Vec::new();
    let mut queue: CowQueue<u64> = CowQueue::default();
    let mut queue_model: VecDeque<u64> = VecDeque::new();
    let mut plain: CowMap<u64, u64, FxBuildHasher> = CowMap::default();
    let mut frozen = None;
    for step in 0..60_000u64 {
        let key = next(5_000);
        match next(4) {
            0 | 1 if !model.iter().any(|(k, _)| *k == key) => {
                map.insert(key, step);
                model.push((key, step));
            }
            2 => {
                let position = model.iter().position(|(k, _)| *k == key);
                let removed = map.swap_remove_full(&key);
                assert_eq!(removed, position.map(|p| {
                    let (k, v) = model.swap_remove(p);
                    (p, k, v)
                }));
            }
            _ => {
                if let Some(value) = map.get_mut(&key) {
                    *value = step;
                    model.iter_mut().find(|(k, _)| *k == key).unwrap().1 = step;
                }
            }
        }
        if next(3) == 0 {
            queue.push_back(step);
            queue_model.push_back(step);
        } else {
            assert_eq!(queue.pop_front(), queue_model.pop_front());
        }
        plain.insert(key, step);
        if step == 30_000 {
            frozen = Some((map.clone(), model.clone(), queue.clone(), queue_model.clone()));
        }
    }
    let pairs = |map: &CowIndexMap<u64, u64, FxBuildHasher>| -> Vec<(u64, u64)> {
        map.iter().map(|(k, v)| (*k, *v)).collect()
    };
    assert_eq!(pairs(&map), model);
    assert_eq!(map.len(), model.len());
    assert!(model.iter().enumerate().all(|(i, (k, v))| map.get_index(i) == Some((k, v))));
    assert!(queue.iter().copied().eq(queue_model.iter().copied()));
    assert!(queue.iter().rev().copied().eq(queue_model.iter().rev().copied()));
    assert_eq!(plain.len(), plain.iter().count());
    let (map, model, queue, queue_model) = frozen.unwrap();
    assert_eq!(pairs(&map), model, "a clone keeps what it saw");
    assert!(queue.iter().copied().eq(queue_model.iter().copied()));
}

/// The graveyard saved and read back: every tombstone, its removal (a replacement's link
/// included), body, prevouts, entry and chunk rate; a file that doesn't read is set aside, and
/// nothing is read in its place.
#[test]
fn graveyard_outlives_a_restart() {
    use crate::stores::tx_graveyard::GraveyardFile;
    let dir = std::env::temp_dir().join(format!("bitview-graveyard-{}", std::process::id()));
    let file = GraveyardFile::new(dir.join("graveyard"));
    let script = ScriptBuf::new_p2wpkh(&bitcoin::WPubkeyHash::from_byte_array([3; 20]));
    // A body as the preparer builds one: weight, size and fee its entry's, sigops counted.
    let body = |seed: u8| {
        let mut tx = spending(seed, &[(txid(90 + seed), Vout::from(0usize))]);
        tx.input[0].prevout = Some(TxOut::from((script.clone(), Sats::from(5_000u64))));
        tx.input[0].sequence = 0xffff_fffd;
        tx.output = vec![TxOut::from((script.clone(), Sats::from(4_000u64)))];
        let raw = bitcoin::Transaction::from(&tx);
        tx.txid = Txid::from(raw.compute_txid());
        tx.total_size = raw.total_size();
        tx.weight = Weight::from(raw.weight());
        tx.fee = Sats::from(1_000u64);
        tx.refresh_sigops();
        (tx, raw)
    };
    let replacer = body(2).0.txid;
    // 4 was replaced by 5, pending when saved: its tree's head goes with it.
    let (head, head_raw) = body(5);
    let (mut graveyard, mut linker) = (TxGraveyard::default(), Linker::default());
    let mut buried = Vec::new();
    for (seed, removal) in [
        (1, TxRemoval::Replaced { by: replacer }),
        (2, TxRemoval::Mined),
        (3, TxRemoval::Vanished),
        (4, TxRemoval::Replaced { by: head.txid }),
    ] {
        let (tx, raw) = body(seed);
        let info = MempoolEntryInfo {
            txid: tx.txid,
            wtxid: raw.compute_wtxid(),
            vsize: VSize::from(tx.weight),
            weight: tx.weight,
            fee: tx.fee,
            first_seen: Timestamp::from(1_700_000_000usize + usize::from(seed)),
        };
        buried.push(tx.txid);
        let record = TxRecord::new(Arc::new(tx), TxEntry::new(&info, raw.total_size() as u64, true));
        graveyard.bury(record, FeeRate::from_milli(1_234 * u64::from(seed)), removal, &mut linker);
    }
    graveyard.set_told(BlockHash::from(bitcoin::BlockHash::from_byte_array([5; 32])), &mut linker);
    let head_info = MempoolEntryInfo {
        txid: head.txid,
        wtxid: head_raw.compute_wtxid(),
        vsize: VSize::from(head.weight),
        weight: head.weight,
        fee: head.fee,
        first_seen: Timestamp::from(1_700_000_005usize),
    };
    let head = TxRecord::new(
        Arc::new(head),
        TxEntry::new(&head_info, head_raw.total_size() as u64, true),
    );
    let head_rate = FeeRate::from_milli(7_777);
    let pending = |txid: &Txid| (*txid == head.entry.txid).then_some((&head, head_rate));
    let taken = std::time::Instant::now();
    file.save(&graveyard, pending, None, taken).unwrap();
    let (loaded, pool) = file.load();
    assert!(pool.is_empty(), "a periodic save carries no pool");
    for txid in &buried {
        let (original, back) = (graveyard.get(txid).unwrap(), loaded.get(txid).unwrap());
        assert_eq!(format!("{:?}", back.removal), format!("{:?}", original.removal));
        assert_eq!(back.chunk_rate, original.chunk_rate);
        assert_eq!(back.record.entry.first_seen, original.record.entry.first_seen);
        assert_eq!(back.record.entry.wtxid, original.record.entry.wtxid);
        assert!(back.record.entry.rbf);
        assert_eq!(back.record.digest, original.record.digest, "the same body, prevouts too");
        assert_eq!(back.removed_at, original.removed_at, "buried when it was, however long ago");
        assert!(back.removed_at <= loaded.told_at().unwrap(), "buried before it was told");
    }
    assert_eq!(
        loaded.predecessors_of(&replacer).map(|(txid, _)| *txid).collect::<Vec<_>>(),
        [buried[0]],
        "the replacement's link"
    );
    assert_eq!(loaded.told(), graveyard.told(), "the tip its blocks were told");
    let restored_head = loaded.get(&head.entry.txid).expect("the pending head");
    assert!(matches!(restored_head.removal, TxRemoval::Vanished));
    assert_eq!(restored_head.chunk_rate, head_rate);
    assert_eq!(restored_head.record.entry.first_seen, head.entry.first_seen);
    assert_eq!(
        loaded.predecessors_of(&head.entry.txid).map(|(txid, _)| *txid).collect::<Vec<_>>(),
        [buried[3]]
    );
    assert_eq!(file.saved_revision(), Some(graveyard.revision()));
    // A clean shutdown's save of the same publication carries the pool, the head pending in it:
    // a restore puts it back, its tombstone exhumed.
    let mut txs = TxStore::default();
    txs.insert(head.tx.clone(), head.entry.clone());
    file.save(&graveyard, pending, Some(&txs.read_only_clone()), taken)
        .unwrap();
    let (loaded, pool) = file.load();
    assert_eq!(pool.len(), 1);
    assert_eq!(pool[0].digest, head.digest, "the same body, prevouts too");
    let state = State::restored(loaded, pool);
    assert!(state.txs.contains(&head.entry.txid));
    assert!(state.graveyard.get(&head.entry.txid).is_none(), "exhumed");
    assert_eq!(state.info.count, 1);
    // No periodic save goes over the shutdown's, even of a later publication.
    file.save(&graveyard, pending, None, taken + std::time::Duration::from_secs(1))
        .unwrap();
    assert_eq!(file.load().1.len(), 1, "the shutdown's pool kept");
    // Corrupt: set aside, an empty graveyard instead.
    std::fs::write(file.path(), b"not a graveyard").unwrap();
    assert_eq!(file.load().0.replacements_recent_first().count(), 0);
    assert!(file.path().with_extension("unread").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

/// A replacement won by a conflict the pool never saw (one a block confirmed): its tree roots at
/// that conflict, unseen (its facts the chain's), whichever of the two is asked about, and the
/// listing shows it though only one version was pending.
#[test]
fn unseen_confirmed_replacers_root_their_trees() {
    let mut state = State::default();
    let replaced = body(1);
    let conflict = txid(9);
    let info = MempoolEntryInfo {
        txid: replaced.txid,
        wtxid: bitcoin::Wtxid::from_byte_array([1; 32]),
        vsize: VSize::from(replaced.weight),
        weight: replaced.weight,
        fee: replaced.fee,
        first_seen: Timestamp::now(),
    };
    let record = TxRecord::new(replaced.clone(), TxEntry::new(&info, 100, true));
    state.graveyard.bury(
        record,
        FeeRate::default(),
        TxRemoval::Replaced { by: conflict },
        &mut state.linker,
    );
    let mut rebuilder = Rebuilder::default();
    rebuilder.tick(&state, None, FeeRate::default(), &[], &RapidHashSet::default(), true);
    let tip = BlockHash::default();
    let read = ReadOnlyState::default()
        .updated(&state, tip, rebuilder.snapshot().clone(), true)
        .expect("published");
    for asked in [replaced.txid, conflict] {
        let root = read.rbf_for_tx(&asked, &tip).unwrap().root.expect("a tree");
        assert_eq!(root.txid, conflict);
        assert!(root.unseen);
        assert!(!root.truncated, "whole");
        assert_eq!(root.replaces.len(), 1);
        assert_eq!(root.replaces[0].txid, replaced.txid);
    }
    let trees = read.recent_rbf_trees(false, &tip).unwrap();
    assert_eq!(trees.len(), 1, "one tree, rooted at the conflict");
    assert_eq!(trees[0].txid, conflict);
}

/// An address's transactions in page order, past the inline vector's spill and back: inserts and
/// removals in any order keep it sorted, and pages read newest first before a cursor.
#[test]
fn address_transactions_keep_page_order() {
    use crate::stores::addr_tracker::{AddrEntry, AddrTx};
    use brk_types::TxidPrefix;
    let tx = |seed: u32| AddrTx {
        first_seen: Timestamp::from((seed % 7) as usize),
        prefix: TxidPrefix::from(&txid((seed % 251) as u8)),
        key: crate::key::TxKey::from(&txid((seed % 251) as u8)),
    };
    let mut entry = AddrEntry::default();
    let mut model: Vec<AddrTx> = Vec::new();
    let mut state = 0x1234_5678u64;
    let mut next = |bound: u64| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state % bound
    };
    for _ in 0..5_000 {
        let candidate = tx(next(120) as u32);
        if next(3) == 0 {
            entry.txs.remove(&candidate);
            model.retain(|t| *t != candidate);
        } else {
            entry.txs.insert(candidate);
            if !model.contains(&candidate) {
                model.push(candidate);
            }
        }
        model.sort_unstable();
        assert_eq!(entry.txs.len(), model.len());
        assert!(entry.txs.iter().copied().eq(model.iter().copied()));
        if let Some(cursor) = model.get(model.len() / 2) {
            let before: Vec<AddrTx> = entry.txs.newest_before(Some(cursor)).unwrap().copied().collect();
            let expected: Vec<AddrTx> = model.iter().filter(|t| *t < cursor).rev().copied().collect();
            assert_eq!(before, expected);
        }
    }
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
    let mut snapshot = Snapshot::build(
        txs,
        indexed[0].clone(),
        false,
        &indexed[1..],
        KeyIndex::default(),
        Chunks::default(),
        FeeRate::default(),
        None,
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

/// BIP125's signaling passes down to unconfirmed descendants, a child that arrived before its
/// signaling parent included, and goes once that parent leaves; one with no signaling ancestor
/// stays final.
#[test]
fn signaling_passes_to_pooled_descendants() {
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
    let id = |tx: &bitcoin::Transaction| Txid::from(tx.compute_txid());
    let info = |tx: &bitcoin::Transaction| MempoolEntryInfo {
        txid: id(tx),
        wtxid: tx.compute_wtxid(),
        vsize: VSize::from(Weight::from(tx.weight())),
        weight: Weight::from(tx.weight()),
        fee: Sats::from(1_000u64),
        first_seen: Timestamp::now(),
    };
    let final_seq = 0xffff_fffe;
    let parent = raw((bitcoin::Txid::from_byte_array([7; 32]), 0), 0);
    let child = raw((parent.compute_txid(), 0), final_seq);
    let grandchild = raw((child.compute_txid(), 0), final_seq);
    let lone = raw((bitcoin::Txid::from_byte_array([8; 32]), 0), final_seq);
    let mut state = State::default();
    let enter = |state: &mut State, txs: &[&bitcoin::Transaction], live: &[Txid]| {
        let entries = txs.iter().map(|tx| info(tx)).collect();
        let bodies = txs.iter().map(|&tx| (id(tx), tx.clone())).collect();
        let diff = preparer::prepare(live, entries, bodies, &Confirmed::default(), state);
        applier::apply(state, &Snapshot::default(), diff, &Confirmed::default());
    };
    // The descendants first (their parent not loaded yet), then the parent.
    enter(&mut state, &[&child, &grandchild, &lone], &[id(&child), id(&grandchild), id(&lone)]);
    let rbf = |state: &State, tx: &bitcoin::Transaction| state.txs.entry(&id(tx)).unwrap().rbf;
    assert!(!rbf(&state, &child) && !rbf(&state, &grandchild));
    let all = [id(&child), id(&grandchild), id(&lone), id(&parent)];
    enter(&mut state, &[&parent], &all);
    assert!(rbf(&state, &parent) && rbf(&state, &child) && rbf(&state, &grandchild));
    assert!(!rbf(&state, &lone), "no signaling ancestor");
    // The parent leaves (mined, say): its descendants signal no more, as Core reads them.
    enter(&mut state, &[], &[id(&child), id(&grandchild), id(&lone)]);
    assert!(!rbf(&state, &child) && !rbf(&state, &grandchild));
}

/// A history past 63 versions comes cut to its newest ones, its root marked `truncated`, in a
/// transaction's own tree and in the recent listing.
#[test]
fn long_histories_come_truncated() {
    let mut state = State::default();
    let conflict = txid(200);
    let versions: Vec<_> = (1..=70u8).map(body).collect();
    for (i, version) in versions.iter().enumerate() {
        let info = MempoolEntryInfo {
            txid: version.txid,
            wtxid: bitcoin::Wtxid::from_byte_array([i as u8; 32]),
            vsize: VSize::from(version.weight),
            weight: version.weight,
            fee: version.fee,
            first_seen: Timestamp::now(),
        };
        let by = versions.get(i + 1).map_or(conflict, |next| next.txid);
        let record = TxRecord::new(version.clone(), TxEntry::new(&info, 100, true));
        state.graveyard.bury(
            record,
            FeeRate::default(),
            TxRemoval::Replaced { by },
            &mut state.linker,
        );
    }
    let mut rebuilder = Rebuilder::default();
    rebuilder.tick(&state, None, FeeRate::default(), &[], &RapidHashSet::default(), true);
    let tip = BlockHash::default();
    let read = ReadOnlyState::default()
        .updated(&state, tip, rebuilder.snapshot().clone(), true)
        .expect("published");
    let depth = |root: &RbfNode| {
        let (mut depth, mut node) = (1, root);
        while let Some(next) = node.replaces.first() {
            depth += 1;
            node = next;
        }
        depth
    };
    let root = read
        .rbf_for_tx(&versions[69].txid, &tip)
        .unwrap()
        .root
        .expect("a tree");
    assert!(root.truncated);
    assert_eq!(depth(&root), 63, "the root and the newest 62 versions");
    assert_eq!(root.replaces[0].txid, versions[69].txid, "newest first");
    // The oldest, 71 versions below its root: the tree comes rooted 62 versions above it.
    let root = read.rbf_for_tx(&versions[0].txid, &tip).unwrap().root.expect("a tree");
    assert!(root.truncated);
    assert_eq!(root.txid, versions[62].txid);
    let mut deepest = &root;
    while let Some(next) = deepest.replaces.first() {
        deepest = next;
    }
    assert_eq!(deepest.txid, versions[0].txid, "the one asked for is in it");
    let trees = read.recent_rbf_trees(false, &tip).unwrap();
    assert_eq!(trees.len(), 1);
    assert!(trees[0].truncated, "listed, cut");
}
