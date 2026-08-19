use brk_error::Result;

use brk_indexer::Indexer;
use brk_types::{
    ChunkInput, CpfpClusterTxIndex, FeeRate, OutPoint, Sats, StoredBool, StoredU64, TxInIndex,
    TxIndex, VSize, linearize,
};
use smallvec::SmallVec;
use vecdb::{
    AnyStoredVec, AnyVec, ColumnId, Exit, PcoVec, ReadableVec, VecIndex, WritableVec, unlikely,
};

use super::super::size;
use super::{CpfpRoleId, Vecs};

#[allow(clippy::too_many_arguments)]
pub fn compute(
    vecs: &mut Vecs,
    indexer: &Indexer,
    input_values: &PcoVec<TxInIndex, Sats>,
    indexes: &bitview_plugin_indexes::Vecs,
    size_vecs: &size::Vecs,
    exit: &Exit,
) -> Result<()> {
    vecs.compute(indexer, input_values, indexes, size_vecs, exit)
}

impl Vecs {
    #[allow(clippy::too_many_arguments)]
    fn compute(
        &mut self,
        indexer: &Indexer,
        input_values: &PcoVec<TxInIndex, Sats>,
        indexes: &bitview_plugin_indexes::Vecs,
        size_vecs: &size::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_lengths = indexer.safe_lengths();

        self.input_value.compute_sum_from_indexes(
            starting_lengths.tx_index,
            &indexer.vecs().transactions.first_txin_index,
            &indexes.tx_index.input_count,
            input_values,
            exit,
        )?;
        self.output_value.compute_sum_from_indexes(
            starting_lengths.tx_index,
            &indexer.vecs().transactions.first_txout_index,
            &indexes.tx_index.output_count,
            &indexer.vecs().outputs.value,
            exit,
        )?;

        self.compute_fees(indexer, indexes, size_vecs, exit)?;

        let vsize_source = &size_vecs.vsize.tx_index;
        let (r1, r2) = rayon::join(
            || {
                self.fee.derive_from_with_skip(
                    indexes,
                    &starting_lengths,
                    &indexer.vecs().transactions.first_tx_index,
                    exit,
                    1,
                )
            },
            || {
                self.effective_fee_rate.derive_from_with_skip_weighted(
                    indexes,
                    &starting_lengths,
                    &indexer.vecs().transactions.first_tx_index,
                    vsize_source,
                    exit,
                    1,
                )
            },
        );
        r1?;
        r2?;

        Ok(())
    }

    fn compute_fees(
        &mut self,
        indexer: &Indexer,
        indexes: &bitview_plugin_indexes::Vecs,
        size_vecs: &size::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let starting_lengths = indexer.safe_lengths();

        let dep_version = self.input_value.version()
            + self.output_value.version()
            + size_vecs.vsize.tx_index.version()
            + indexer.vecs().inputs.outpoint.version()
            + indexer.vecs().transactions.first_tx_index.version()
            + indexer.vecs().transactions.first_txin_index.version()
            + indexes.height.tx_index_count.version();

        self.fee
            .tx_index
            .validate_computed_version_or_reset(dep_version)?;
        self.fee_rate
            .validate_computed_version_or_reset(dep_version)?;
        self.effective_fee_rate
            .tx_index
            .validate_computed_version_or_reset(dep_version)?;
        self.cpfp_flags_source
            .validate_computed_version_or_reset(dep_version)?;
        self.count
            .cumulative
            .validate_computed_version_or_reset(dep_version)?;

        let target = self
            .input_value
            .len()
            .min(self.output_value.len())
            .min(size_vecs.vsize.tx_index.len());
        let tx_len = self
            .fee
            .tx_index
            .len()
            .min(self.fee_rate.len())
            .min(self.effective_fee_rate.tx_index.len())
            .min(self.cpfp_flags_source.len())
            .min(starting_lengths.tx_index.to_usize());
        let max_height = indexer
            .vecs()
            .transactions
            .first_tx_index
            .len()
            .min(indexes.height.tx_index_count.len());
        let next_height = if tx_len >= target {
            max_height
        } else {
            indexes
                .tx_heights
                .get_shared(TxIndex::from(tx_len))
                .unwrap()
                .to_usize()
        };
        let count_len = self.count.cumulative.len().min(max_height);
        let start_height = count_len.min(next_height);
        if start_height >= max_height {
            return Ok(());
        }

        let start_tx = indexer
            .vecs()
            .transactions
            .first_tx_index
            .collect_one_at(start_height)
            .unwrap()
            .to_usize();
        self.fee
            .tx_index
            .truncate_if_needed(TxIndex::from(start_tx))?;
        self.fee_rate.truncate_if_needed(TxIndex::from(start_tx))?;
        self.effective_fee_rate
            .tx_index
            .truncate_if_needed(TxIndex::from(start_tx))?;
        self.cpfp_flags_source
            .truncate_if_needed(TxIndex::from(start_tx))?;
        self.count.truncate_if_needed_at(start_height)?;

        let mut tx_count = indexes.height.tx_index_count.cursor();
        let mut next_block_input = indexer.vecs().inputs.first_txin_index.cursor();
        tx_count.advance(start_height);
        next_block_input.advance(start_height + 1);

        let mut input_values = Vec::new();
        let mut output_values = Vec::new();
        let mut vsizes = Vec::new();
        let mut txin_starts = Vec::new();
        let mut outpoints = Vec::new();
        let mut fees = Vec::new();
        let mut cluster = Cluster::default();
        let mut first_tx = start_tx;

        for h in start_height..max_height {
            let n = u64::from(tx_count.next().unwrap()) as usize;

            if first_tx + n > target {
                break;
            }

            // Batch read all per-tx data for this block
            self.input_value
                .collect_range_into_at(first_tx, first_tx + n, &mut input_values);
            self.output_value
                .collect_range_into_at(first_tx, first_tx + n, &mut output_values);
            size_vecs
                .vsize
                .tx_index
                .collect_range_into_at(first_tx, first_tx + n, &mut vsizes);
            indexer
                .vecs()
                .transactions
                .first_txin_index
                .collect_range_into_at(first_tx, first_tx + n, &mut txin_starts);
            let input_begin = txin_starts[0].to_usize();
            let input_end = if h + 1 < max_height {
                next_block_input.next().unwrap().to_usize()
            } else {
                indexer.vecs().inputs.outpoint.len()
            };
            indexer.vecs().inputs.outpoint.collect_range_into_at(
                input_begin,
                input_end,
                &mut outpoints,
            );

            // Compute fee + fee_rate per tx
            fees.clear();
            fees.reserve(n);
            for j in 0..n {
                let fee = if unlikely(input_values[j].is_max()) {
                    Sats::ZERO
                } else {
                    input_values[j] - output_values[j]
                };
                self.fee.tx_index.push(fee);
                self.fee_rate.push(FeeRate::from((fee, vsizes[j])));
                fees.push(fee);
            }

            // Effective fee rate via same-block CPFP clustering
            cluster_fee_rates(
                &txin_starts,
                &outpoints,
                input_begin,
                first_tx,
                &fees,
                &vsizes,
                &mut cluster,
            );
            let mut parent_count = 0;
            let mut child_count = 0;
            for ((&effective, &fee), &vsize) in cluster.rates.iter().zip(&fees).zip(&vsizes) {
                let (is_parent, is_child) = cpfp_roles(effective, FeeRate::from((fee, vsize)));
                parent_count += is_parent as u64;
                child_count += is_child as u64;
                self.effective_fee_rate.tx_index.push(effective);
                self.cpfp_flags_source
                    .push([StoredBool::from(is_parent), StoredBool::from(is_child)]);
            }
            self.count
                .push_block(CpfpRoleId::from_fn(|role| match role {
                    CpfpRoleId::Parent => StoredU64::from(parent_count),
                    CpfpRoleId::Child => StoredU64::from(child_count),
                }));

            if h % 1_000 == 0 {
                let _lock = exit.lock();
                self.fee.tx_index.write()?;
                self.fee_rate.write()?;
                self.effective_fee_rate.tx_index.write()?;
                self.cpfp_flags_source.write()?;
                self.count.write()?;
            }

            first_tx += n;
        }

        let _lock = exit.lock();
        self.fee.tx_index.write()?;
        self.fee_rate.write()?;
        self.effective_fee_rate.tx_index.write()?;
        self.cpfp_flags_source.write()?;
        self.count.write()?;

        Ok(())
    }
}

/// Computes SFL chunk rates for each same-block dependency component.
fn cluster_fee_rates(
    txin_starts: &[TxInIndex],
    outpoints: &[OutPoint],
    outpoint_base: usize,
    first_tx: usize,
    fees: &[Sats],
    vsizes: &[VSize],
    cluster: &mut Cluster,
) {
    let n = fees.len();
    cluster.rates.clear();
    cluster.rates.extend(
        fees.iter()
            .zip(vsizes)
            .map(|(&fee, &vsize)| FeeRate::from((fee, vsize))),
    );
    cluster.parents.clear();
    cluster.parents.resize_with(n, SmallVec::new);
    cluster.roots.clear();
    cluster.roots.extend(0..n);
    cluster.members.clear();
    cluster.local_index.clear();
    cluster.local_index.resize(n, usize::MAX);

    for child in 0..n {
        let mut parents: SmallVec<[usize; 2]> =
            same_block_parents(child, txin_starts, outpoints, outpoint_base, first_tx, n).collect();
        parents.sort_unstable();
        parents.dedup();
        for &parent in &parents {
            union(&mut cluster.roots, child, parent);
        }
        cluster.parents[child] = parents;
    }

    for tx in 0..n {
        cluster.members.push((root(&mut cluster.roots, tx), tx));
    }
    cluster.members.sort_unstable();

    let mut start = 0;
    while start < n {
        let component_root = cluster.members[start].0;
        let end = cluster.members[start..]
            .partition_point(|&(candidate, _)| candidate == component_root)
            + start;
        if end - start > 1 {
            linearize_component(
                &cluster.members[start..end],
                &cluster.parents,
                fees,
                vsizes,
                &mut cluster.rates,
                &mut cluster.local_index,
                &mut cluster.local_parents,
            );
        }
        start = end;
    }
}

#[allow(clippy::too_many_arguments)]
fn linearize_component(
    members: &[(usize, usize)],
    parents: &[SmallVec<[usize; 2]>],
    fees: &[Sats],
    vsizes: &[VSize],
    rates: &mut [FeeRate],
    local_index: &mut [usize],
    local_parents: &mut Vec<SmallVec<[CpfpClusterTxIndex; 2]>>,
) {
    for (local, &(_, tx)) in members.iter().enumerate() {
        local_index[tx] = local;
    }

    local_parents.clear();
    local_parents.extend(members.iter().map(|&(_, tx)| {
        parents[tx]
            .iter()
            .map(|&parent| CpfpClusterTxIndex::from(local_index[parent] as u32))
            .collect()
    }));

    let inputs: Vec<ChunkInput<'_>> = members
        .iter()
        .enumerate()
        .map(|(local, &(_, tx))| ChunkInput {
            fee: fees[tx],
            vsize: vsizes[tx],
            parents: local_parents[local].as_slice(),
        })
        .collect();

    for chunk in linearize(&inputs) {
        for local in chunk.txs {
            rates[members[u32::from(local) as usize].1] = chunk.feerate;
        }
    }
}

fn union(roots: &mut [usize], left: usize, right: usize) {
    let left = root(roots, left);
    let right = root(roots, right);
    if left != right {
        roots[right] = left;
    }
}

fn root(roots: &mut [usize], node: usize) -> usize {
    let mut root = node;
    while roots[root] != root {
        root = roots[root];
    }

    let mut current = node;
    while roots[current] != current {
        let next = roots[current];
        roots[current] = root;
        current = next;
    }
    root
}

fn same_block_parents<'a>(
    tx: usize,
    txin_starts: &'a [TxInIndex],
    outpoints: &'a [OutPoint],
    outpoint_base: usize,
    first_tx: usize,
    tx_count: usize,
) -> impl Iterator<Item = usize> + 'a {
    let start = txin_starts[tx].to_usize() - outpoint_base;
    let end = txin_starts
        .get(tx + 1)
        .map_or(outpoints.len(), |index| index.to_usize() - outpoint_base);

    outpoints[start..end].iter().filter_map(move |outpoint| {
        let parent = outpoint.tx_index().to_usize();
        (parent >= first_tx && parent < first_tx + tx_count).then(|| parent - first_tx)
    })
}

fn cpfp_roles(effective: FeeRate, raw: FeeRate) -> (bool, bool) {
    (effective > raw, effective < raw)
}

#[derive(Default)]
struct Cluster {
    rates: Vec<FeeRate>,
    parents: Vec<SmallVec<[usize; 2]>>,
    roots: Vec<usize>,
    members: Vec<(usize, usize)>,
    local_index: Vec<usize>,
    local_parents: Vec<SmallVec<[CpfpClusterTxIndex; 2]>>,
}

#[cfg(test)]
mod tests {
    use brk_types::{FeeRate, OutPoint, Sats, TxInIndex, TxIndex, VSize, Vout};

    use super::{Cluster, cluster_fee_rates, cpfp_roles};

    #[test]
    fn marks_actual_cpfp_roles() {
        let mut cluster = Cluster::default();
        cluster_fee_rates(
            &[TxInIndex::from(0usize), TxInIndex::from(1usize)],
            &[
                OutPoint::COINBASE,
                OutPoint::new(TxIndex::from(10usize), Vout::ZERO),
            ],
            0,
            10,
            &[Sats::new(100), Sats::new(200)],
            &[VSize::new(100), VSize::new(100)],
            &mut cluster,
        );

        assert_eq!(cluster.rates, [FeeRate::new(1.5), FeeRate::new(1.5)]);
        assert_eq!(
            [
                cpfp_roles(cluster.rates[0], FeeRate::new(1.0)),
                cpfp_roles(cluster.rates[1], FeeRate::new(2.0)),
            ],
            [(true, false), (false, true)]
        );
    }

    #[test]
    fn keeps_independent_transaction_rates_separate() {
        let mut cluster = Cluster::default();
        cluster_fee_rates(
            &[TxInIndex::from(0usize), TxInIndex::from(1usize)],
            &[
                OutPoint::COINBASE,
                OutPoint::new(TxIndex::from(9usize), Vout::ZERO),
            ],
            0,
            10,
            &[Sats::new(100), Sats::new(300)],
            &[VSize::new(100), VSize::new(100)],
            &mut cluster,
        );

        assert_eq!(cluster.rates, [FeeRate::new(1.0), FeeRate::new(3.0)]);
        assert_eq!(
            [
                cpfp_roles(cluster.rates[0], FeeRate::new(1.0)),
                cpfp_roles(cluster.rates[1], FeeRate::new(3.0)),
            ],
            [(false, false), (false, false)]
        );
    }

    #[test]
    fn linearizes_shared_parent_branches_independently_of_sibling_order() {
        let txin_starts = [
            TxInIndex::from(0usize),
            TxInIndex::from(1usize),
            TxInIndex::from(2usize),
        ];
        let outpoints = [
            OutPoint::COINBASE,
            OutPoint::new(TxIndex::from(10usize), Vout::ZERO),
            OutPoint::new(TxIndex::from(10usize), Vout::ZERO),
        ];
        let vsizes = [VSize::new(100); 3];
        let mut cluster = Cluster::default();

        cluster_fee_rates(
            &txin_starts,
            &outpoints,
            0,
            10,
            &[Sats::ZERO, Sats::ZERO, Sats::new(3_000)],
            &vsizes,
            &mut cluster,
        );
        assert_eq!(
            cluster.rates,
            [FeeRate::new(15.0), FeeRate::new(0.0), FeeRate::new(15.0)]
        );

        cluster_fee_rates(
            &txin_starts,
            &outpoints,
            0,
            10,
            &[Sats::ZERO, Sats::new(3_000), Sats::ZERO],
            &vsizes,
            &mut cluster,
        );
        assert_eq!(
            cluster.rates,
            [FeeRate::new(15.0), FeeRate::new(15.0), FeeRate::new(0.0)]
        );
    }
}
