use brk_cpfp::{Cluster, find_seed_chunk};
use brk_types::{
    CPFP_CHAIN_LIMIT, CpfpCluster, CpfpClusterTx, CpfpClusterTxIndex, CpfpEntry, CpfpInfo, FeeRate,
    Height, SigOps, TxIndex, Txid, VSize, Weight,
};
use rustc_hash::{FxBuildHasher, FxHashSet};
use smallvec::SmallVec;
use vecdb::{ReadableVec, VecIndex};

use bitview_primitives::Lengths;

use crate::{OptionData, Query, ResolvedConfirmedTx, Result};

struct WalkResult {
    /// Each `graph` member's txid and weight, by local index.
    members: Vec<(Txid, Weight)>,
    seed_local: CpfpClusterTxIndex,
    ancestors: Vec<TxIndex>,
    descendants: Vec<TxIndex>,
}

impl Query {
    pub(crate) fn confirmed_cpfp_resolved(
        &self,
        transaction: ResolvedConfirmedTx,
    ) -> Result<CpfpInfo> {
        let read = self.read_indexer()?;
        let (_, seed, height) = read.revalidate_confirmed_tx(transaction)?;
        let safe = read.pin().lengths();
        let mut graph = Cluster::default();
        // Count views end at their source's visible length, bounded by the read's pin.
        let walk = self.walk_same_block_cluster(seed, height, safe, &mut graph)?;
        let ancestors = self.resolve_entries(&walk.ancestors)?;
        let descendants = self.resolve_entries(&walk.descendants)?;
        let sigops = self
            .indexer()
            .vecs()
            .transactions
            .total_sigop_cost
            .collect_one(seed)
            .data()?
            .into();
        drop(read);

        Ok(build_cpfp_info(
            &mut graph,
            &walk.members,
            walk.seed_local,
            ancestors,
            descendants,
            sigops,
        ))
    }

    fn resolve_entries(&self, indexes: &[TxIndex]) -> Result<Vec<CpfpEntry>> {
        let indexer = self.indexer();
        let plugins = self.plugins();
        let mut weight = indexer.vecs().transactions.weight.cursor();
        let mut fee = plugins.transactions.fees.fee.tx.cursor();
        let txid = indexer.vecs().transactions.txid.reader();

        indexes
            .iter()
            .map(|index| {
                let position = index.to_usize();
                Ok(CpfpEntry {
                    txid: txid.get(*index),
                    fee: fee.get(position).data()?,
                    weight: weight.get(position).data()?.into(),
                })
            })
            .collect()
    }

    fn walk_same_block_cluster(
        &self,
        seed: TxIndex,
        height: Height,
        safe: Lengths,
        graph: &mut Cluster,
    ) -> Result<WalkResult> {
        let indexer = self.indexer();
        let plugins = self.plugins();
        let first_tx = &indexer.vecs().transactions.first_tx_index;
        let block_first = first_tx.collect_one(height).data()?;
        let next_height = height.incremented();
        let block_end = if next_height < safe.height {
            first_tx.collect_one(next_height).data()?
        } else {
            safe.tx_index
        };

        let mut first_txin = indexer.vecs().transactions.first_txin_index.cursor();
        let mut input_count = plugins.mappings.tx_index.input_count.cursor();
        let mut outpoint = indexer.vecs().inputs.outpoint.cursor();
        let first_txout = indexer.vecs().transactions.first_txout_index.reader();
        let mut output_count = plugins.mappings.tx_index.output_count.cursor();
        let spent = plugins.outputs.spent.spending_txin_index.reader();
        let mut spending_tx = indexer.vecs().inputs.tx_index.cursor();

        let mut parents_of = |tx: TxIndex| -> Result<SmallVec<[TxIndex; 2]>> {
            let position = tx.to_usize();
            let first = usize::from(first_txin.get(position).data()?);
            let count = u64::from(input_count.get(position).data()?) as usize;
            let mut parents = SmallVec::new();
            for input in first..first + count {
                let parent = outpoint.get(input).data()?;
                if !parent.is_coinbase()
                    && parent.tx_index() >= block_first
                    && parent.tx_index() < block_end
                {
                    parents.push(parent.tx_index());
                }
            }
            parents.sort_unstable();
            parents.dedup();
            Ok(parents)
        };

        let mut children_of = |tx: TxIndex| -> Result<SmallVec<[TxIndex; 2]>> {
            let position = tx.to_usize();
            let first = usize::from(first_txout.try_get_at(position).data()?.get());
            let count = u64::from(output_count.get(position).data()?) as usize;
            let mut children = SmallVec::new();
            for output in first..first + count {
                let input = spent.try_get_at(output).data()?.get();
                // The spend index changes in place: unpublished spends read as unspent.
                if input.is_unspent() || input >= safe.txin_index {
                    continue;
                }
                let child = spending_tx.get(usize::from(input)).data()?;
                if child >= block_first && child < block_end {
                    children.push(child);
                }
            }
            children.sort_unstable();
            children.dedup();
            Ok(children)
        };

        let block_tx_count = block_end.to_usize() - block_first.to_usize();
        let mut component =
            walk_component(seed, &mut parents_of, &mut children_of, block_tx_count)?;
        component.sort_unstable();
        let ancestors = walk_direction(seed, &mut parents_of, CPFP_CHAIN_LIMIT)?;
        let descendants = walk_direction(seed, &mut children_of, CPFP_CHAIN_LIMIT)?;

        // Same-block parents precede their children, so the sorted component is topological
        // and a member's local index is its sorted position.
        let seed_local = CpfpClusterTxIndex::from(component.partition_point(|&tx| tx < seed) as u32);
        let mut weight = indexer.vecs().transactions.weight.cursor();
        let mut fee = plugins.transactions.fees.fee.tx.cursor();
        let txid = indexer.vecs().transactions.txid.reader();
        graph.clear();
        let mut members = Vec::with_capacity(component.len());
        for &tx in &component {
            let position = tx.to_usize();
            let tx_weight = Weight::from(weight.get(position).data()?);
            graph.push(
                fee.get(position).data()?,
                VSize::from(tx_weight),
                parents_of(tx)?.iter().filter_map(|parent| {
                    component
                        .binary_search(parent)
                        .ok()
                        .map(|local| CpfpClusterTxIndex::from(local as u32))
                }),
            );
            members.push((txid.get(tx), tx_weight));
        }

        Ok(WalkResult {
            members,
            seed_local,
            ancestors,
            descendants,
        })
    }
}

fn walk_component(
    seed: TxIndex,
    parents: &mut impl FnMut(TxIndex) -> Result<SmallVec<[TxIndex; 2]>>,
    children: &mut impl FnMut(TxIndex) -> Result<SmallVec<[TxIndex; 2]>>,
    limit: usize,
) -> Result<Vec<TxIndex>> {
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
    next: &mut impl FnMut(TxIndex) -> Result<SmallVec<[TxIndex; 2]>>,
    limit: usize,
) -> Result<Vec<TxIndex>> {
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
    graph: &mut Cluster,
    members: &[(Txid, Weight)],
    seed_local: CpfpClusterTxIndex,
    ancestors: Vec<CpfpEntry>,
    descendants: Vec<CpfpEntry>,
    sigops: SigOps,
) -> CpfpInfo {
    let seed_position = u32::from(seed_local) as usize;
    let (fee, vsize) = (graph.fee(seed_position), graph.vsize(seed_position));
    let raw_rate = FeeRate::from((fee, vsize));
    let best_descendant = descendants
        .iter()
        .max_by_key(|entry| FeeRate::from((entry.fee, entry.weight)))
        .cloned();

    let (cluster, effective_fee_per_vsize) = if members.len() == 1 {
        (None, raw_rate)
    } else {
        let chunks = graph.chunks();
        let (chunk_index, rate) = find_seed_chunk(&chunks, seed_local, raw_rate);
        let txs = members
            .iter()
            .enumerate()
            .map(|(local, &(txid, weight))| CpfpClusterTx {
                txid,
                weight,
                fee: graph.fee(local),
                parents: graph.parents(local).to_vec(),
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
        fee,
        vsize,
        adjusted_vsize: sigops.adjust_vsize(vsize),
        cluster,
    }
}
