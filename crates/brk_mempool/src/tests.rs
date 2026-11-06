use bitcoin::hashes::Hash;
use brk_cpfp::Cluster;
use brk_types::{FeeRate, Sats, SigOps, Txid, VSize, Weight};
use smallvec::SmallVec;

use crate::snapshot::{
    SnapTx, TxIndex, cluster,
    partition::{Chunks, partition},
};

/// Random pools (chains, fans, sigop-heavy transactions), the last transaction a chunk no block
/// can hold, packed after a block 0 that took an ancestor-closed prefix: each member once, after
/// its parents, every block but the last within Core's budgets, the oversized chunk in the last,
/// and packing ends whatever is left (a remainder of the oversized chunk alone included).
#[test]
fn projected_blocks_follow_dependencies_and_budgets() {
    for (n, taken) in [(30_000, 2_000), (60, 10), (1, 0)] {
        let txs = pool(n);
        let excluded: Vec<bool> = (0..n).map(|i| i < taken).collect();
        let blocks = partition(&txs, &chunks(&txs), &excluded, 7);
        assert!(blocks.len() <= 7);
        let mut placed = excluded.clone();
        for (b, block) in blocks.iter().enumerate() {
            let (mut weight, mut sigops) = (Weight::default(), SigOps::ZERO);
            for &(m, _) in block {
                let t = &txs[m.as_usize()];
                assert!(!placed[m.as_usize()], "a transaction in two blocks");
                assert!(t.parents.iter().all(|p| placed[p.as_usize()]), "before a parent");
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
        assert!(oversized, "the oversized chunk waits for the last");
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
            SnapTx {
                txid: Txid::from(bitcoin::Txid::from_byte_array(bytes)),
                fee: Sats::from(next(40_000)),
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
    for i in 0..n {
        for p in txs[i].parents.clone() {
            txs[p.as_usize()].children.push(TxIndex::from(i));
        }
    }
    txs
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
