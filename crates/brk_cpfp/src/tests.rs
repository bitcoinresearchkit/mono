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


/// Exhaustive closure oracle for small signed graphs. It makes no assumptions about chunk
/// connectivity, tie order or solver tier. Consecutive equal-rate chunks form one diagram edge.
fn signed_brute(fees: &[i64], weights: &[u64], parents: &[Vec<usize>], mut remaining: u32) -> Vec<(i128, u64)> {
    let mut result = Vec::new();
    while remaining != 0 {
        let mut best: Option<(i128, u64, u32)> = None;
        let mut subset = remaining;
        while subset != 0 {
            if (0..fees.len()).all(|i| subset & (1 << i) == 0 || parents[i].iter().all(|&p| remaining & (1 << p) == 0 || subset & (1 << p) != 0)) {
                let (fee, weight) = (0..fees.len()).filter(|&i| subset & (1 << i) != 0)
                    .fold((0, 0), |(f, w), i| (f + i128::from(fees[i]), w + weights[i]));
                if best.is_none_or(|(f, w, _)| fee * i128::from(w) > f * i128::from(weight)) {
                    best = Some((fee, weight, subset));
                }
            }
            subset = (subset - 1) & remaining;
        }
        let (fee, weight, subset) = best.unwrap();
        result.push((fee, weight));
        remaining &= !subset;
    }
    signed_diagram(&result)
}

fn signed_diagram(chunks: &[(i128, u64)]) -> Vec<(i128, u64)> {
    let mut result: Vec<(i128, u64)> = Vec::new();
    for &(fee, weight) in chunks {
        match result.last_mut() {
            Some((f, w)) if *f * i128::from(weight) == fee * i128::from(*w) => {
                *f += fee;
                *w += weight;
            }
            _ => result.push((fee, weight)),
        }
    }
    result
}

fn signed_chunks(fees: &[i64], weights: &[u64], parents: &[Vec<usize>], heuristic: bool) -> Vec<(Vec<usize>, i128, u64)> {
    let mut cluster = Cluster::default();
    for i in 0..fees.len() {
        cluster.push_ranked(Sats::from(1u64), fees[i], Weight::from(weights[i]), i as u32,
            parents[i].iter().map(|&p| CpfpClusterTxIndex::from(p as u32)));
    }
    let mut chunks = Vec::new();
    let mut placed = vec![false; fees.len()];
    let mut emit = |members: &[CpfpClusterTxIndex], _: FeeRate| {
        let members: Vec<usize> = members.iter().map(|&m| u32::from(m) as usize).collect();
        for &i in &members {
            assert!(!placed[i]);
            assert!(parents[i].iter().all(|&p| placed[p]));
            placed[i] = true;
        }
        let (fee, weight) = members.iter().fold((0, 0), |(f, w), &i| (f + i128::from(fees[i]), w + weights[i]));
        if let Some((_, f, w)) = chunks.last() {
            assert!(fee * i128::from(*w) <= f * i128::from(weight));
        }
        chunks.push((members, fee, weight));
    };
    if heuristic { cluster.linearize_ancestor_sets(&mut emit); } else { cluster.linearize(&mut emit); }
    assert!(placed.into_iter().all(|p| p));
    chunks
}

#[test]
fn signed_diagrams_and_every_exact_suffix_match_closure_oracle() {
    let mut state = 0x47b1_2de9_83a6_c057u64;
    let mut next = |n: u64| {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state % n
    };
    let amounts = [i64::MIN, i64::MAX, 0, -1, 1, -4_000_000_000_000_000, 4_000_000_000_000_000];
    for round in 0..3_000 {
        let n = 1 + next(9) as usize;
        let fees: Vec<i64> = (0..n).map(|_| amounts[next(amounts.len() as u64) as usize]).collect();
        let weights: Vec<u64> = (0..n).map(|_| 100 + next(900)).collect();
        let tree = round % 2 == 0;
        let parents: Vec<Vec<usize>> = (0..n).map(|i| {
            if tree { if i == 0 { vec![] } else { vec![next(i as u64) as usize] } }
            else { (0..i).filter(|_| next(3) == 0).collect() }
        }).collect();
        for heuristic in [false, true] {
            let chunks = signed_chunks(&fees, &weights, &parents, heuristic);
            if heuristic && !tree { continue; } // The heuristic promises optimality on trees.
            let mut remaining = (1u32 << n) - 1;
            for boundary in 0..chunks.len() {
                let suffix: Vec<(i128, u64)> = chunks[boundary..].iter().map(|&(_, f, w)| (f, w)).collect();
                assert_eq!(signed_diagram(&suffix), signed_brute(&fees, &weights, &parents, remaining), "round {round}, heuristic {heuristic}, boundary {boundary}");
                for &i in &chunks[boundary].0 { remaining &= !(1 << i); }
            }
        }
    }
}

#[test]
fn wide_signed_chains_follow_the_prefix_reference_at_tier_boundaries() {
    // Each negative/positive pair has zero fee. Every legal prefix has rate <= zero, so
    // repeated two-member zero-rate chunks, followed by the final negative, are optimal.
    for n in [64, 65, 130, 4_097] {
        let fees: Vec<i64> = (0..n).map(|i| if i % 2 == 0 { -i64::MAX } else { i64::MAX }).collect();
        let weights = vec![400; n];
        let parents: Vec<Vec<usize>> = (0..n).map(|i| if i == 0 { vec![] } else { vec![i - 1] }).collect();
        let chunks = signed_chunks(&fees, &weights, &parents, false);
        let diagram = signed_diagram(&chunks.iter().map(|&(_, f, w)| (f, w)).collect::<Vec<_>>());
        let mut expected = vec![(0, (n / 2 * 800) as u64)];
        if n % 2 != 0 { expected.push((-i128::from(i64::MAX), 400)); }
        assert_eq!(diagram, expected);
        for fee in [i64::MIN, i64::MAX] {
            let fees = vec![fee; n];
            let weights: Vec<u64> = (0..n).map(|i| 400 + if fee < 0 { i } else { n - i } as u64).collect();
            let chunks = signed_chunks(&fees, &weights, &parents, false);
            assert_eq!(chunks.len(), 1);
            assert_eq!(chunks[0].1, i128::from(fee) * n as i128);
            assert_eq!(chunks[0].2, weights.iter().sum::<u64>());
        }
    }
}

#[test]
fn signed_rate_comparison_handles_products_beyond_i128() {
    use crate::compare_feerates;
    // Scaling both numerators preserves the order. The small products fit i128; the
    // scaled products deliberately overflow and exercise the continued-fraction fallback.
    for i in 1..10_000u64 {
        let a = (i128::from((i * 7919) % 2001) - 1000, Weight::from(u64::MAX - i * 47));
        let b = (i128::from((i * 3797) % 2001) - 1000, Weight::from(u64::MAX / (i % 13 + 1)));
        let scale = 1i128 << 100;
        assert_eq!(compare_feerates(a, b), compare_feerates((a.0 * scale, a.1), (b.0 * scale, b.1)));
    }
    for fee in [i128::MIN, i128::MIN + 1, -1, 0, 1, i128::MAX - 1, i128::MAX] {
        assert!(compare_feerates((fee, Weight::from(u64::MAX)), (fee, Weight::from(u64::MAX))).is_eq());
        if fee > 0 {
            assert!(compare_feerates((fee, Weight::from(u64::MAX - 1)), (fee, Weight::from(u64::MAX))).is_gt());
        } else if fee < 0 {
            assert!(compare_feerates((fee, Weight::from(u64::MAX - 1)), (fee, Weight::from(u64::MAX))).is_lt());
        }
    }
}
