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
    MempoolBlockSource, ReadOnlyState, State,
    snapshot::Rebuilder,
    state::{Ancestry, Block},
    snapshot::{
        SnapTx, Snapshot, TxIndex,
        builder::{ChunkCache, KeyIndex},
        cluster,
        partition::{Chunks, partition},
    },
    steps::{Confirmed, applier, preparer::{self, TxAddition}, prevouts},
    state::TxEntry,
    stores::{TxGraveyard, TxStore, Upkeep, tx_store::TxRecord},
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
        let projection = partition(&txs, &chunks(&txs), &excluded, 7, &mut ChunkCache::default());
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
/// whole, h's output being pooled); behind the reader, the pool goes without what the newer
/// blocks mined (m), conflicted (c) or what descends from a conflicted one (d), while a mined
/// one's child (k) and the unrelated (u) stay pending; past a reorg that left the pool's tip,
/// without what's in limbo (h and hc, spending from what only the left block mined).
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
    let hash = |seed: u8| BlockHash::from(bitcoin::BlockHash::from_byte_array([seed; 32]));
    // The tip's block mined what h spends, after its coinbase.
    let spent_by_h = Txid::from(confirmed(83).txid);
    let tip_block = Block { hash: tip, txids: Arc::from([txid(9), spent_by_h]) };
    state.ancestry.follow(tip, &[Block::unread(hash(4)), tip_block]);
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
    let behind = read.view_behind(reader, 0, spends).unwrap();
    let again = read
        .view_behind(reader, 0, || -> brk_error::Result<[_; 0]> {
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
    let utxos = read.addr_pending_utxos(&addr, usize::MAX, &behind).unwrap();
    assert_eq!(utxos.created.len(), 2, "k's and u's outputs");
    let mut spends = utxos.spends();
    assert!(spends.spent(&id(&m), Vout::from(0usize)));
    assert_ne!(spends.token(), utxos.token, "the spenders met are part of the token");
    assert!(
        read.view_at(&BlockHash::from(bitcoin::BlockHash::from_byte_array([1; 32])))
            .is_err(),
        "another tip"
    );
    // Past a reorg that left the pool's tip block: what only it mined (its coinbase aside) and
    // what the pool holds spending from it (h, and hc from h) are in limbo, out of the view.
    let none = || -> brk_error::Result<[((Txid, Vout), Txid); 0]> { Ok([]) };
    let forked = read.view_behind(hash(3), 1, none).unwrap();
    for (txid, limbo) in [(spent_by_h, true), (id(&h), true), (id(&hc), true), (txid(9), false), (id(&u), false)] {
        assert_eq!(forked.in_limbo(&txid), limbo);
    }
    assert!(!read.is_pending(&id(&hc), &forked).unwrap());
    assert!(read.is_pending(&id(&u), &forked).unwrap());
    // The ancestry: back to the fork after a reorg, to the fork itself when one only shortened
    // the chain.
    let mut ancestry = Ancestry::default();
    ancestry.follow(hash(2), &[1, 2].map(|seed| Block::unread(hash(seed))));
    ancestry.follow(hash(5), &[1, 5].map(|seed| Block::unread(hash(seed))));
    assert_eq!(ancestry.blocks(), [1, 5].map(|seed| Block::unread(hash(seed))));
    ancestry.follow(hash(1), &[Block::unread(hash(1))]);
    assert_eq!(ancestry.blocks(), [Block::unread(hash(1))]);

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
        let (mut graveyard, mut upkeep) = (TxGraveyard::default(), Upkeep::default());
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
                    graveyard.bury(body(seed, next(40)), FeeRate::default(), removal, &mut upkeep);
                    if let TxRemoval::Replaced { by } = removal {
                        graveyard.exhume(&by, &mut upkeep);
                    }
                }
                5 => graveyard.exhume(&txid(seed), &mut upkeep),
                6 => {
                    let spends = (0..next(6))
                        .map(|_| (outpoint(next(40)), txid(next(48) as u8)))
                        .collect::<Vec<_>>();
                    let mined = (0..next(4))
                        .map(|_| txid(next(48) as u8))
                        .chain(spends.iter().map(|(_, by)| *by))
                        .collect::<Vec<_>>();
                    let disconnected = (0..next(3)).map(|_| txid(next(48) as u8)).collect::<Vec<_>>();
                    let listed = next(48) as u8;
                    let live = |id: &Txid| *id == txid(listed);
                    graveyard.confirm(&Confirmed::of(mined, spends, disconnected), live, &mut upkeep);
                }
                7 => graveyard.set_confirmed_tip(
                    BlockHash::from(bitcoin::BlockHash::from_byte_array([next(4) as u8; 32])),
                    &mut upkeep,
                ),
                8 => {
                    graveyard.index_vanished(&mut upkeep);
                    let unloaded = (0..next(3)).map(|_| txid(next(48) as u8)).collect();
                    graveyard.end_cycle(&unloaded, &mut upkeep);
                }
                9 => {
                    graveyard.exhume(&txid(seed), &mut upkeep);
                    graveyard.relink(&spending(seed, &[outpoint(next(40))]), &mut upkeep);
                }
                _ => {
                    hours += next(30);
                    let listed = next(48) as u8;
                    let live = |id: &Txid| *id == txid(listed);
                    let at = start + Duration::from_secs(hours * 3_600);
                    graveyard.evict_old(at, live, &mut upkeep);
                }
            }
            assert!(graveyard.consistent(&upkeep), "graveyard invariants, history {history}");
        }
    }
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
    let (mut graveyard, mut upkeep) = (TxGraveyard::default(), Upkeep::default());
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
        graveyard.bury(record, FeeRate::from_milli(1_234 * u64::from(seed)), removal, &mut upkeep);
    }
    graveyard.set_confirmed_tip(BlockHash::from(bitcoin::BlockHash::from_byte_array([5; 32])), &mut upkeep);
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
        assert!(back.removed_at <= loaded.confirmed_at().unwrap(), "buried before it was told");
    }
    assert!(loaded.consistent(&Upkeep::of(&loaded)), "its bookkeeping rebuilt");
    assert_eq!(
        loaded.predecessors_of(&replacer).map(|(txid, _)| *txid).collect::<Vec<_>>(),
        [buried[0]],
        "the replacement's link"
    );
    assert_eq!(loaded.confirmed_tip(), graveyard.confirmed_tip(), "the tip its blocks were told");
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

/// Paging agrees with rust-bitcoin's ordered map, including absent/removed cursors and old
/// publications held across a dense-store removal. Display-hex order is deliberately different.
#[test]
fn mempool_txid_pages_follow_binary_order_and_publications() {
    use std::collections::BTreeSet;
    use bitcoin::hashes::sha256d;

    let id = |n: u64| Txid::from(bitcoin::Txid::from_byte_array(
        sha256d::Hash::hash(&n.to_le_bytes()).to_byte_array(),
    ));
    let mut state = State::default();
    let mut model = BTreeSet::new();
    for n in 0..12_000 {
        let txid = id(n);
        let tx = Arc::new(Transaction { txid, ..(*body(1)).clone() });
        let info = MempoolEntryInfo {
            txid,
            wtxid: bitcoin::Wtxid::from_byte_array([0; 32]),
            vsize: VSize::from(tx.weight),
            weight: tx.weight,
            fee: tx.fee,
            first_seen: Timestamp::from(n as usize),
        };
        let entry = TxEntry::new(&info, 100, false);
        state.insert(tx, entry);
        model.insert(bitcoin::Txid::from(txid));
    }
    let mut rebuilder = Rebuilder::default();
    let read = publish_pool(&state, &ReadOnlyState::default(), &mut rebuilder);
    let mut equivalent = read.clone();
    equivalent.pool = Some(Arc::new(crate::state::Pool::new(
        &state, BlockHash::default(), read.pool().unwrap().graph.clone(), Some(read.pool().unwrap()),
    )));
    assert!(read.pool().unwrap().sorted_txids.get().is_none());
    assert!(read.txids_page(None, 0).unwrap().is_empty());
    assert!(read.pool().unwrap().sorted_txids.get().is_none(), "zero page must not sort");
    let all: Vec<_> = model.iter().copied().map(Txid::from).collect();
    assert_eq!(read.txids_page(None, usize::MAX).unwrap(), all);
    assert!(Arc::ptr_eq(read.pool().unwrap().sorted_txids.get().unwrap(),
        equivalent.pool().unwrap().sorted_txids.get().unwrap()), "late initialization shared across publications");
    assert!(Arc::ptr_eq(&read.txids_json().unwrap(), &equivalent.txids_json().unwrap()));
    let full_page = read.txids_page(None, 10_000).unwrap();
    assert_eq!(full_page, &all[..10_000]);
    assert_eq!(serde_json::to_vec(&full_page).unwrap().len(), 670_001);
    for n in (0..2_048).chain(12_000..14_048) {
        let cursor = id(n);
        let limit = n as usize % 43;
        let expected: Vec<_> = model.range((std::ops::Bound::Excluded(bitcoin::Txid::from(cursor)), std::ops::Bound::Unbounded))
            .take(limit).copied().map(Txid::from).collect();
        assert_eq!(read.txids_page(Some(&cursor), limit).unwrap(), expected);
    }
    let removed = all[all.len() / 2];
    state.remove(&crate::key::TxKey::from(&removed));
    model.remove(&bitcoin::Txid::from(removed));
    let next = publish_pool(&state, &read, &mut rebuilder);
    assert_eq!(read.txids_page(None, usize::MAX).unwrap(), all, "retained publication");
    assert_eq!(next.txids_page(None, usize::MAX).unwrap(), model.iter().copied().map(Txid::from).collect::<Vec<_>>());
    let expected: Vec<_> = model.range((std::ops::Bound::Excluded(bitcoin::Txid::from(removed)), std::ops::Bound::Unbounded))
        .take(100).copied().map(Txid::from).collect();
    assert_eq!(next.txids_page(Some(&removed), 100).unwrap(), expected);
}

#[test]
fn cached_bodies_follow_retained_publications_through_burial_and_expiry() {
    use std::time::Duration;

    assert!(ReadOnlyState::default().cached_transaction(&txid(7)).is_err());
    for removal in [TxRemoval::Mined, TxRemoval::Vanished, TxRemoval::Replaced { by: txid(8) }] {
        let mut state = State::default();
        let tx = body(7);
        let first_seen = Timestamp::new(17);
        let entry = MempoolEntryInfo {
            txid: tx.txid, wtxid: bitcoin::Wtxid::from_byte_array([7; 32]),
            vsize: VSize::from(tx.weight), weight: tx.weight, fee: tx.fee, first_seen,
        };
        state.insert(tx.clone(), TxEntry::new(&entry, tx.total_size as u64, false));
        let mut rebuilder = Rebuilder::default();
        let live = publish_pool(&state, &ReadOnlyState::default(), &mut rebuilder);
        let (captured, _) = live.cached_transaction(&tx.txid).unwrap().unwrap();
        assert!(Arc::ptr_eq(&captured.tx, &tx));
        assert_eq!(captured.first_seen, first_seen);
        assert!(live.cached_transaction(&txid(99)).unwrap().is_none());

        let record = state.remove(&crate::key::TxKey::from(&tx.txid));
        let burial_rate = FeeRate::from_milli(1_234);
        state.graveyard.bury(record, burial_rate, removal, &mut state.upkeep);
        let buried_at = state.graveyard.get(&tx.txid).unwrap().removed_at;
        let buried = publish_pool(&state, &live, &mut rebuilder);
        let (retained, rate) = buried.cached_transaction(&tx.txid).unwrap().unwrap();
        assert!(Arc::ptr_eq(&retained.tx, &tx));
        assert_eq!(retained.first_seen, first_seen);
        assert_eq!(rate, burial_rate);
        assert!(!retained.tx.status.confirmed, "cached status is the retained mempool view");

        state.graveyard.evict_old(buried_at + Duration::from_secs(48 * 3_600 + 1), |_| false, &mut state.upkeep);
        let expired = publish_pool(&state, &buried, &mut rebuilder);
        assert!(expired.cached_transaction(&tx.txid).unwrap().is_none());
        assert!(live.cached_transaction(&tx.txid).unwrap().is_some());
        assert!(buried.cached_transaction(&tx.txid).unwrap().is_some(), "old readers keep their complete observation");
        assert!(Arc::ptr_eq(&captured.tx, &retained.tx));
    }
}

fn publish_pool(state: &State, previous: &ReadOnlyState, rebuilder: &mut Rebuilder) -> ReadOnlyState {
    rebuilder.tick(state, None, FeeRate::default(), &[], &RapidHashSet::default(), true);
    previous.updated(state, BlockHash::default(), rebuilder.snapshot().clone(), true).unwrap()
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

