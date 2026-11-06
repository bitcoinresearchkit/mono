use brk_types::{CpfpClusterTxIndex, FeeRate, Sats, Weight};

use crate::Cluster;

/// The feerate diagram: chunks as (fee, vsize), consecutive equal feerates summed (the one
/// invariant of an optimal linearization; equal-feerate chunks may split any way).
fn diagram(chunks: &[(u64, u64)]) -> Vec<(u64, u64)> {
    let mut merged: Vec<(u64, u64)> = Vec::new();
    for &(fee, vsize) in chunks {
        match merged.last_mut() {
            Some(last)
                if u128::from(last.0) * u128::from(vsize)
                    == u128::from(fee) * u128::from(last.1) =>
            {
                last.0 += fee;
                last.1 += vsize;
            }
            _ => merged.push((fee, vsize)),
        }
    }
    merged
}

/// Optimal chunks by brute force: repeatedly the best feerate closure of what remains.
fn brute(fees: &[u64], weights: &[u64], parents: &[Vec<usize>]) -> Vec<(u64, u64)> {
    let n = fees.len();
    let mut remaining: u32 = (1 << n) - 1;
    let mut chunks = Vec::new();
    while remaining != 0 {
        let mut best: Option<(u64, u64)> = None;
        let mut best_set = 0;
        let mut sub = remaining;
        while sub != 0 {
            let closed = (0..n).all(|i| {
                sub & (1 << i) == 0
                    || parents[i]
                        .iter()
                        .all(|&p| remaining & (1 << p) == 0 || sub & (1 << p) != 0)
            });
            if closed {
                let (fee, vsize) = (0..n)
                    .filter(|&i| sub & (1 << i) != 0)
                    .fold((0, 0), |(f, v), i| (f + fees[i], v + weights[i]));
                let better = best.is_none_or(|(bf, bv)| {
                    u128::from(fee) * u128::from(bv) > u128::from(bf) * u128::from(vsize)
                });
                if better {
                    best = Some((fee, vsize));
                    best_set = sub;
                }
            }
            sub = (sub - 1) & remaining;
        }
        chunks.push(best.unwrap());
        remaining &= !best_set;
    }
    chunks
}

/// The chunks, each checked: after its parents, every member once, its published feerate its fee
/// over its weight's exact quarter, and never paying more than the chunk before.
fn ours(fees: &[u64], weights: &[u64], parents: &[Vec<usize>]) -> Vec<(u64, u64)> {
    checked(fees, weights, parents, |cluster, emit| cluster.linearize(emit))
}

/// [`ours`] through the ancestor-set tier (the greedy, then PostLinearize), whatever the size.
fn ancestor_tier(fees: &[u64], weights: &[u64], parents: &[Vec<usize>]) -> Vec<(u64, u64)> {
    checked(fees, weights, parents, |cluster, emit| {
        cluster.linearize_ancestor_sets(emit)
    })
}

fn checked(
    fees: &[u64],
    weights: &[u64],
    parents: &[Vec<usize>],
    linearize: impl FnOnce(&mut Cluster, &mut dyn FnMut(&[CpfpClusterTxIndex], FeeRate)),
) -> Vec<(u64, u64)> {
    let mut cluster = Cluster::default();
    for i in 0..fees.len() {
        cluster.push(
            Sats::from(fees[i]),
            Weight::from(weights[i]),
            parents[i]
                .iter()
                .map(|&p| CpfpClusterTxIndex::from(p as u32)),
        );
    }
    let mut chunks: Vec<(u64, u64)> = Vec::new();
    let mut placed = vec![false; fees.len()];
    linearize(&mut cluster, &mut |txs, rate| {
        let members: Vec<usize> = txs.iter().map(|&tx| u32::from(tx) as usize).collect();
        // In order: each member after its parents, earlier chunks' or this one's earlier members.
        for &i in &members {
            assert!(!placed[i], "a member in two chunks");
            assert!(
                parents[i].iter().all(|&p| placed[p]),
                "a member before its parents"
            );
            placed[i] = true;
        }
        let chunk = members
            .iter()
            .fold((0, 0), |(f, w), &i| (f + fees[i], w + weights[i]));
        assert_eq!(
            rate,
            FeeRate::from((Sats::from(chunk.0), Weight::from(chunk.1)))
        );
        if let Some(&(fee, weight)) = chunks.last() {
            let later = u128::from(chunk.0) * u128::from(weight);
            let earlier = u128::from(fee) * u128::from(chunk.1);
            assert!(later <= earlier, "a chunk paying more than the one before");
        }
        chunks.push(chunk);
    });
    assert!(placed.iter().all(|&p| p), "every member in a chunk");
    chunks
}

/// The optimal linearization's feerate diagram, against every closure enumerated.
#[test]
fn linearization_is_optimal() {
    // The ancestor-set greedy's miss: P/A/B pay 7, Q 6, D 1 (it picked Q first).
    let (fees, weights) = ([100, 1000, 1000, 600, 100], [100; 5]);
    let parents = vec![vec![], vec![0], vec![0], vec![], vec![0, 3]];
    assert_eq!(
        diagram(&ours(&fees, &weights, &parents)),
        vec![(2100, 300), (600, 100), (100, 100)]
    );

    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    let mut next = |bound: u64| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state % bound
    };
    for _ in 0..20_000 {
        let n = 1 + next(10) as usize;
        let fees: Vec<u64> = (0..n).map(|_| next(5) * next(2_000)).collect();
        let weights: Vec<u64> = (0..n).map(|_| 60 + next(400)).collect();
        let parents: Vec<Vec<usize>> = (0..n)
            .map(|i| (0..i).filter(|_| next(4) == 0).collect())
            .collect();
        assert_eq!(
            diagram(&ours(&fees, &weights, &parents)),
            diagram(&brute(&fees, &weights, &parents)),
            "fees {fees:?} weights {weights:?} parents {parents:?}"
        );
    }
}

/// PostLinearize's guarantee: over trees (each transaction one parent at most, or one child at
/// most) the ancestor-set tier's chunks are optimal, as every closure enumerated finds.
#[test]
fn ancestor_tier_is_optimal_on_trees() {
    let mut state = 0x2545_F491_4F6C_DD1Du64;
    let mut next = |bound: u64| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state % bound
    };
    for round in 0..20_000 {
        let n = 1 + next(10) as usize;
        let fees: Vec<u64> = (0..n).map(|_| next(5) * next(2_000)).collect();
        let weights: Vec<u64> = (0..n).map(|_| 60 + next(400)).collect();
        // Out-trees: one parent at most. In-trees: one child at most (each parent's single child).
        let parents: Vec<Vec<usize>> = if round % 2 == 0 {
            (0..n)
                .map(|i| match i {
                    0 => vec![],
                    _ if next(4) == 0 => vec![],
                    _ => vec![next(i as u64) as usize],
                })
                .collect()
        } else {
            let mut has_child = vec![false; n];
            (0..n)
                .map(|child| {
                    let parents: Vec<usize> = (0..child)
                        .filter(|&parent| !has_child[parent] && next(3) == 0)
                        .collect();
                    for &parent in &parents {
                        has_child[parent] = true;
                    }
                    parents
                })
                .collect()
        };
        assert_eq!(
            diagram(&ancestor_tier(&fees, &weights, &parents)),
            diagram(&brute(&fees, &weights, &parents)),
            "fees {fees:?} weights {weights:?} parents {parents:?}"
        );
    }
}

/// Past 4,096 members, fees at the bound still chunk exactly: a chain whose descendants pay more
/// than its root together, though their sum is past 64 bits, chunk with the root.
#[test]
fn chunk_sums_stay_exact_past_64_bits() {
    let max = 2_100_000_000_000_000u64;
    let n = 5_002;
    let mut cluster = Cluster::default();
    for i in 0..n {
        let modified = match i {
            0 => (max / 5_000 * 4_999) as i64,
            1 => 0,
            _ => max as i64,
        };
        let parents = (i > 0).then(|| CpfpClusterTxIndex::from((i - 1) as u32));
        cluster.push_ranked(Sats::ZERO, modified, Weight::from(400u64), parents);
    }
    let mut chunks = Vec::new();
    cluster.linearize(|txs, _| chunks.push(txs.len()));
    assert_eq!(chunks, [n as usize], "one chunk, the root's feerate below the rest's");
}

/// Each tier, at its bounds: valid chunks (checked by `ours`) for random sparse components.
#[test]
fn every_tier_linearizes() {
    let mut state = 0xD1B5_4A32_D192_ED03u64;
    let mut next = |bound: u64| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state % bound
    };
    for n in [64, 65, 4_096, 4_097] {
        let fees: Vec<u64> = (0..n).map(|_| next(20_000)).collect();
        let weights: Vec<u64> = (0..n).map(|_| 240 + next(4_000)).collect();
        let parents: Vec<Vec<usize>> = (0..n)
            .map(|i| {
                let count = if i == 0 { 0 } else { next(3) };
                let mut ps: Vec<usize> = (0..count).map(|_| next(i as u64) as usize).collect();
                ps.sort_unstable();
                ps.dedup();
                ps
            })
            .collect();
        ours(&fees, &weights, &parents);
    }
}

/// A negative modified fee ranks the package as Core does: a parent paying 1,000 sats with a
/// -2,000 delta and its child paying 3,000 chunk together at 2,000 sats over 800 WU (10 sat/vB),
/// below a lone 12 sat/vB transaction; served fees stay what they paid.
#[test]
fn negative_modified_fees_rank_signed() {
    let mut cluster = Cluster::default();
    cluster.push_ranked(Sats::from(1_000u64), -1_000, Weight::from(400u64), []);
    cluster.push_ranked(
        Sats::from(3_000u64),
        3_000,
        Weight::from(400u64),
        [CpfpClusterTxIndex::from(0)],
    );
    let mut chunks = Vec::new();
    cluster.linearize(|txs, rate| chunks.push((txs.len(), rate)));
    assert_eq!(
        chunks,
        [(
            2,
            FeeRate::from((Sats::from(2_000u64), Weight::from(800u64)))
        )]
    );
    assert!(chunks[0].1 < FeeRate::from((Sats::from(1_200u64), Weight::from(400u64))));
    assert_eq!(cluster.fee(0), Sats::from(1_000u64));
}
