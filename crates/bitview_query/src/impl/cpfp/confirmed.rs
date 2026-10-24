use brk_cpfp::Cluster;
use brk_types::{
    CPFP_CHAIN_LIMIT, ConfirmedCpfp, Cpfp, CpfpClusterTxIndex, CpfpEntry, Height, SigOps, TxIndex,
    Txid, VSize, Weight,
};
use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;
use vecdb::{ReadableVec, VecIndex};

use bitview_plugin_indexer::SafeLengths;

use crate::{Error, OptionData, Query, ResolvedConfirmedTx, Result};

struct WalkResult {
    graph: Cluster,
    /// Each `graph` member's txid and weight, by local index.
    members: Vec<(Txid, Weight)>,
    seed_local: CpfpClusterTxIndex,
    ancestors: Vec<CpfpEntry>,
    descendants: Vec<CpfpEntry>,
}

impl Query {
    pub(crate) fn confirmed_cpfp_resolved(
        &self,
        transaction: ResolvedConfirmedTx,
    ) -> Result<Cpfp> {
        let read = self.read_indexer()?;
        let (_, seed, height) = read.revalidate_confirmed_tx(transaction)?;
        // Count views end at their source's visible length, bounded by the read's pin.
        let walk = self.walk_same_block_cluster(seed, height, read.pin())?;
        drop(read);

        // Without relatives in its block, mempool.space answers `{"ancestors":[]}`.
        let Some(mut walk) = walk else {
            return Ok(Cpfp::EMPTY);
        };
        let (cluster, effective_fee_per_vsize) = walk
            .graph
            .project(walk.seed_local, |local| walk.members[local]);
        Ok(Cpfp::Confirmed(ConfirmedCpfp {
            ancestors: walk.ancestors,
            descendants: walk.descendants,
            effective_fee_per_vsize,
            cluster: cluster.ok_or(Error::Internal("a related transaction has no cluster"))?,
        }))
    }

    /// The seed's same-block cluster; none when it has no relatives in its block.
    fn walk_same_block_cluster(
        &self,
        seed: TxIndex,
        height: Height,
        pin: &SafeLengths,
    ) -> Result<Option<WalkResult>> {
        let indexer = self.indexer();
        let plugins = self.plugins();
        let safe = pin.lengths();
        let (first, count) = self.block_tx_range(height, pin)?;
        let block_first = TxIndex::from(first);
        let block_end = TxIndex::from(first + count);

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

        let component = walk_component(seed, &mut parents_of, &mut children_of)?;
        if component.len() == 1 {
            return Ok(None);
        }
        let ancestors = walk_direction(seed, |tx| &component[&tx].parents);
        let descendants = walk_direction(seed, |tx| &component[&tx].children);

        // Same-block parents precede their children, so the sorted component is topological
        // and a member's local index is its sorted position.
        let mut sorted: Vec<TxIndex> = component.keys().copied().collect();
        sorted.sort_unstable();
        let seed_local = CpfpClusterTxIndex::from(sorted.partition_point(|&tx| tx < seed) as u32);
        let mut weight = indexer.vecs().transactions.weight.cursor();
        let mut sigops = indexer.vecs().transactions.total_sigop_cost.cursor();
        let mut fee = plugins.transactions.fees.fee.tx.cursor();
        let txid = indexer.vecs().transactions.txid.reader();
        let mut graph = Cluster::default();
        let mut members = Vec::with_capacity(sorted.len());
        for &tx in &sorted {
            let position = tx.to_usize();
            let tx_weight = Weight::from(weight.get(position).data()?);
            let tx_sigops = SigOps::from(sigops.get(position).data()?);
            // Rates over the policy-adjusted size, as the mempool's.
            graph.push(
                fee.get(position).data()?,
                tx_sigops.adjust_vsize(VSize::from(tx_weight)),
                component[&tx].parents.iter().map(|parent| {
                    let local = sorted
                        .binary_search(parent)
                        .expect("a member's parent is in its component");
                    CpfpClusterTxIndex::from(local as u32)
                }),
            );
            members.push((txid.get(tx), tx_weight));
        }
        // Ancestors and descendants are members: their entries come from what was just read.
        let entries = |txs: Vec<TxIndex>| -> Vec<CpfpEntry> {
            txs.into_iter()
                .map(|tx| {
                    let local = sorted
                        .binary_search(&tx)
                        .expect("a relative is a component member");
                    let (txid, weight) = members[local];
                    CpfpEntry {
                        txid,
                        fee: graph.fee(local),
                        weight,
                    }
                })
                .collect()
        };

        Ok(Some(WalkResult {
            ancestors: entries(ancestors),
            descendants: entries(descendants),
            graph,
            members,
            seed_local,
        }))
    }
}

/// A member's same-block neighbors.
struct Edges {
    parents: SmallVec<[TxIndex; 2]>,
    children: SmallVec<[TxIndex; 2]>,
}

/// The seed's same-block dependency component, each member's edges read once.
fn walk_component(
    seed: TxIndex,
    parents: &mut impl FnMut(TxIndex) -> Result<SmallVec<[TxIndex; 2]>>,
    children: &mut impl FnMut(TxIndex) -> Result<SmallVec<[TxIndex; 2]>>,
) -> Result<FxHashMap<TxIndex, Edges>> {
    let mut component = FxHashMap::default();
    let mut stack = vec![seed];
    while let Some(tx) = stack.pop() {
        if component.contains_key(&tx) {
            continue;
        }
        let edges = Edges {
            parents: parents(tx)?,
            children: children(tx)?,
        };
        stack.extend(edges.parents.iter().chain(&edges.children).copied());
        component.insert(tx, edges);
    }
    Ok(component)
}

/// Up to [`CPFP_CHAIN_LIMIT`] transactions reachable from `seed` through `next`.
fn walk_direction<'a>(
    seed: TxIndex,
    next: impl Fn(TxIndex) -> &'a SmallVec<[TxIndex; 2]>,
) -> Vec<TxIndex> {
    let mut visited = FxHashSet::default();
    visited.insert(seed);
    let mut members = Vec::new();
    let mut stack = next(seed).to_vec();

    while let Some(tx) = stack.pop() {
        if !visited.insert(tx) {
            continue;
        }
        members.push(tx);
        if members.len() == CPFP_CHAIN_LIMIT {
            break;
        }
        stack.extend_from_slice(next(tx));
    }
    members
}
