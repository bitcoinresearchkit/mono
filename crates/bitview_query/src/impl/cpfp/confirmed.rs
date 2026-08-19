use brk_error::OptionData;
use brk_types::{
    CPFP_CHAIN_LIMIT, ChunkInput, CpfpCluster, CpfpClusterTx, CpfpClusterTxIndex, CpfpEntry,
    CpfpInfo, FeeRate, Height, Sats, TxInIndex, TxIndex, Txid, VSize, Weight, find_seed_chunk,
    linearize,
};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use smallvec::SmallVec;
use vecdb::{ReadableVec, VecIndex};

use crate::Query;

struct WalkResult {
    members: Vec<(TxIndex, SmallVec<[CpfpClusterTxIndex; 2]>)>,
    seed_local: CpfpClusterTxIndex,
    ancestors: Vec<TxIndex>,
    descendants: Vec<TxIndex>,
}

struct Member {
    txid: Txid,
    fee: Sats,
    weight: Weight,
    vsize: VSize,
    parents: SmallVec<[CpfpClusterTxIndex; 2]>,
}

impl Query {
    fn confirmed_cpfp(&self, txid: &Txid) -> brk_error::Result<CpfpInfo> {
        let seed = crate::r#impl::tx::resolve_tx_index(self, txid)?;
        let height = crate::r#impl::tx::confirmed_status_height(self, seed)?;
        let walk = self.walk_same_block_cluster(seed, height)?;
        let members = self.resolve_members(&walk.members)?;
        let ancestors = self.resolve_entries(&walk.ancestors)?;
        let descendants = self.resolve_entries(&walk.descendants)?;
        let sigops = self
            .indexer()
            .vecs()
            .transactions
            .total_sigop_cost
            .collect_one(seed)
            .data()?;

        Ok(build_cpfp_info(
            &members,
            walk.seed_local,
            ancestors,
            descendants,
            sigops,
        ))
    }

    fn resolve_members(
        &self,
        members: &[(TxIndex, SmallVec<[CpfpClusterTxIndex; 2]>)],
    ) -> brk_error::Result<Vec<Member>> {
        let indexer = self.indexer();
        let computer = self.computer();
        let mut weight = indexer.vecs().transactions.weight.cursor();
        let mut fee = computer.transactions.fees.fee.tx_index.cursor();
        let txid = indexer.vecs().transactions.txid.reader();

        members
            .iter()
            .map(|(index, parents)| {
                let position = index.to_usize();
                let weight = weight.get(position).data()?;
                Ok(Member {
                    txid: txid.get(*index),
                    fee: fee.get(position).data()?,
                    weight,
                    vsize: VSize::from(weight),
                    parents: parents.clone(),
                })
            })
            .collect()
    }

    fn resolve_entries(&self, indexes: &[TxIndex]) -> brk_error::Result<Vec<CpfpEntry>> {
        let indexer = self.indexer();
        let computer = self.computer();
        let mut weight = indexer.vecs().transactions.weight.cursor();
        let mut fee = computer.transactions.fees.fee.tx_index.cursor();
        let txid = indexer.vecs().transactions.txid.reader();

        indexes
            .iter()
            .map(|index| {
                let position = index.to_usize();
                Ok(CpfpEntry {
                    txid: txid.get(*index),
                    fee: fee.get(position).data()?,
                    weight: weight.get(position).data()?,
                })
            })
            .collect()
    }

    fn walk_same_block_cluster(
        &self,
        seed: TxIndex,
        height: Height,
    ) -> brk_error::Result<WalkResult> {
        let indexer = self.indexer();
        let computer = self.computer();
        let safe = self.safe_lengths();
        let first_tx = &indexer.vecs().transactions.first_tx_index;
        let block_first = first_tx.collect_one(height).data()?;
        let next_height = height.incremented();
        let block_end = if next_height < safe.height {
            first_tx.collect_one(next_height).data()?
        } else {
            safe.tx_index
        };

        let mut first_txin = indexer.vecs().transactions.first_txin_index.cursor();
        let mut input_count = computer.indexes.tx_index.input_count.cursor();
        let mut outpoint = indexer.vecs().inputs.outpoint.cursor();
        let first_txout = indexer
            .vecs()
            .transactions
            .first_txout_index
            .reader()
            .cursor();
        let mut output_count = computer.indexes.tx_index.output_count.cursor();
        let spent = computer.outputs.spent.txin_index.reader().cursor();
        let mut spending_tx = indexer.vecs().inputs.tx_index.cursor();

        let mut parents_of = |tx: TxIndex| -> brk_error::Result<SmallVec<[TxIndex; 2]>> {
            let position = tx.to_usize();
            let first = usize::from(first_txin.get(position).data()?);
            let count = u64::from(input_count.get(position).data()?) as usize;
            let mut parents = SmallVec::new();
            for input in first..first + count {
                let parent = outpoint.get(input).data()?;
                if !parent.is_coinbase()
                    && parent.tx_index() >= block_first
                    && parent.tx_index() < block_end
                    && !parents.contains(&parent.tx_index())
                {
                    parents.push(parent.tx_index());
                }
            }
            parents.sort_unstable();
            Ok(parents)
        };

        let mut children_of = |tx: TxIndex| -> brk_error::Result<SmallVec<[TxIndex; 2]>> {
            let position = tx.to_usize();
            let first = usize::from(first_txout.get(position).data()?);
            let count = u64::from(output_count.get(position).data()?) as usize;
            let mut children = SmallVec::new();
            for output in first..first + count {
                let input = spent.get(output).data()?;
                if input == TxInIndex::UNSPENT {
                    continue;
                }
                let child = spending_tx.get(usize::from(input)).data()?;
                if child >= block_first && child < block_end && !children.contains(&child) {
                    children.push(child);
                }
            }
            children.sort_unstable();
            Ok(children)
        };

        let block_tx_count = block_end.to_usize() - block_first.to_usize();
        let mut component =
            walk_component(seed, &mut parents_of, &mut children_of, block_tx_count)?;
        component.sort_unstable();
        let ancestors = walk_direction(seed, &mut parents_of, CPFP_CHAIN_LIMIT)?;
        let descendants = walk_direction(seed, &mut children_of, CPFP_CHAIN_LIMIT)?;

        let local_of: FxHashMap<TxIndex, CpfpClusterTxIndex> = component
            .iter()
            .enumerate()
            .map(|(local, &tx)| (tx, CpfpClusterTxIndex::from(local as u32)))
            .collect();
        let seed_local = local_of[&seed];
        let members = component
            .into_iter()
            .map(|tx| {
                let parents = parents_of(tx)?
                    .iter()
                    .filter_map(|parent| local_of.get(parent).copied())
                    .collect();
                Ok((tx, parents))
            })
            .collect::<brk_error::Result<_>>()?;

        Ok(WalkResult {
            members,
            seed_local,
            ancestors,
            descendants,
        })
    }
}

#[inline]
pub fn confirmed_cpfp(query: &Query, txid: &Txid) -> brk_error::Result<CpfpInfo> {
    query.confirmed_cpfp(txid)
}

fn walk_component(
    seed: TxIndex,
    parents: &mut impl FnMut(TxIndex) -> brk_error::Result<SmallVec<[TxIndex; 2]>>,
    children: &mut impl FnMut(TxIndex) -> brk_error::Result<SmallVec<[TxIndex; 2]>>,
    limit: usize,
) -> brk_error::Result<Vec<TxIndex>> {
    let mut visited = FxHashSet::with_capacity_and_hasher(limit, FxBuildHasher);
    visited.insert(seed);
    let mut members = Vec::with_capacity(limit);
    members.push(seed);
    let mut stack = vec![seed];

    while let Some(tx) = stack.pop() {
        for neighbor in parents(tx)?.into_iter().chain(children(tx)?) {
            if visited.insert(neighbor) {
                if members.len() == limit {
                    return Ok(members);
                }
                members.push(neighbor);
                stack.push(neighbor);
            }
        }
    }
    Ok(members)
}

fn walk_direction(
    seed: TxIndex,
    next: &mut impl FnMut(TxIndex) -> brk_error::Result<SmallVec<[TxIndex; 2]>>,
    limit: usize,
) -> brk_error::Result<Vec<TxIndex>> {
    let mut visited = FxHashSet::with_capacity_and_hasher(limit + 1, FxBuildHasher);
    visited.insert(seed);
    let mut members = Vec::with_capacity(limit);
    let mut stack = next(seed)?.into_vec();

    while let Some(tx) = stack.pop() {
        if !visited.insert(tx) {
            continue;
        }
        members.push(tx);
        if members.len() == limit {
            break;
        }
        stack.extend(next(tx)?);
    }
    Ok(members)
}

fn build_cpfp_info(
    members: &[Member],
    seed_local: CpfpClusterTxIndex,
    ancestors: Vec<CpfpEntry>,
    descendants: Vec<CpfpEntry>,
    sigops: brk_types::SigOps,
) -> CpfpInfo {
    let seed_position = u32::from(seed_local) as usize;
    let seed = &members[seed_position];
    let raw_rate = FeeRate::from((seed.fee, seed.vsize));
    let best_descendant = descendants
        .iter()
        .max_by_key(|entry| FeeRate::from((entry.fee, entry.weight)))
        .cloned();

    let (cluster, effective_fee_per_vsize) = if members.len() == 1 {
        (None, raw_rate)
    } else {
        let inputs: Vec<ChunkInput<'_>> = members
            .iter()
            .map(|member| ChunkInput {
                fee: member.fee,
                vsize: member.vsize,
                parents: member.parents.as_slice(),
            })
            .collect();
        let chunks = linearize(&inputs);
        let (chunk_index, rate) = find_seed_chunk(&chunks, seed_local, raw_rate);
        let txs = members
            .iter()
            .map(|member| CpfpClusterTx {
                txid: member.txid,
                weight: member.weight,
                fee: member.fee,
                parents: member.parents.iter().copied().collect(),
            })
            .collect();
        (
            Some(CpfpCluster {
                txs,
                chunks,
                chunk_index,
            }),
            rate,
        )
    };

    CpfpInfo {
        ancestors,
        best_descendant,
        descendants,
        effective_fee_per_vsize,
        sigops,
        fee: seed.fee,
        vsize: seed.vsize,
        adjusted_vsize: sigops.adjust_vsize(seed.vsize),
        cluster,
    }
}

#[cfg(test)]
mod tests {
    use brk_types::TxIndex;
    use vecdb::VecIndex;

    use super::{walk_component, walk_direction};

    fn adjacent(
        graph: &[Vec<usize>],
        tx: TxIndex,
    ) -> brk_error::Result<smallvec::SmallVec<[TxIndex; 2]>> {
        Ok(graph[tx.to_usize()]
            .iter()
            .copied()
            .map(TxIndex::from)
            .collect())
    }

    #[test]
    fn component_includes_siblings_but_directional_walk_does_not() {
        let parents = vec![vec![], vec![0], vec![0]];
        let children = vec![vec![1, 2], vec![], vec![]];
        let seed = TxIndex::from(1usize);
        let mut parent = |tx| adjacent(&parents, tx);
        let mut child = |tx| adjacent(&children, tx);

        let mut component = walk_component(seed, &mut parent, &mut child, 64).unwrap();
        component.sort_unstable();
        assert_eq!(component, [0usize, 1, 2].map(TxIndex::from));

        assert_eq!(
            walk_direction(seed, &mut parent, 25).unwrap(),
            [TxIndex::from(0usize)]
        );
        assert!(walk_direction(seed, &mut child, 25).unwrap().is_empty());
    }
}
