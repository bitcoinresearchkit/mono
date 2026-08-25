use brk_types::{
    ChunkInput, CpfpClusterTxIndex, FeeRate, OutPoint, Sats, TxInIndex, VSize, linearize,
};
use smallvec::SmallVec;
use vecdb::{VecIndex, unlikely};

#[derive(Default)]
pub struct Block {
    first_tx: usize,
    pub input_begin: usize,
    pub input_values: Vec<Sats>,
    pub output_values: Vec<Sats>,
    pub fees: Vec<Sats>,
    pub fee_rates: Vec<FeeRate>,
    pub effective_fee_rates: Vec<FeeRate>,
    pub vsizes: Vec<VSize>,
    pub txin_starts: Vec<TxInIndex>,
    pub outpoints: Vec<OutPoint>,
    cluster: Cluster,
}

impl Block {
    pub fn reset(&mut self, first_tx: usize) {
        self.first_tx = first_tx;
        self.input_values.clear();
        self.output_values.clear();
        self.fees.clear();
        self.fee_rates.clear();
        self.vsizes.clear();
        self.txin_starts.clear();
        self.outpoints.clear();
    }

    pub fn compute(&mut self) {
        debug_assert_eq!(self.input_values.len(), self.output_values.len());
        debug_assert_eq!(self.input_values.len(), self.vsizes.len());
        debug_assert_eq!(self.input_values.len(), self.txin_starts.len());

        self.fees.reserve(self.input_values.len());
        self.fee_rates.reserve(self.input_values.len());
        for ((&input, &output), &vsize) in self
            .input_values
            .iter()
            .zip(&self.output_values)
            .zip(&self.vsizes)
        {
            let fee = if unlikely(input.is_max()) {
                Sats::ZERO
            } else {
                input - output
            };
            self.fees.push(fee);
            self.fee_rates.push(FeeRate::from((fee, vsize)));
        }

        self.cluster.compute(
            &self.txin_starts,
            &self.outpoints,
            self.input_begin,
            self.first_tx,
            &self.fees,
            &self.vsizes,
            &self.fee_rates,
            &mut self.effective_fee_rates,
        );
    }
}

#[derive(Default)]
struct Cluster {
    parents: Vec<SmallVec<[usize; 2]>>,
    roots: Vec<usize>,
    members: Vec<(usize, usize)>,
    local_index: Vec<usize>,
    local_parents: Vec<SmallVec<[CpfpClusterTxIndex; 2]>>,
}

impl Cluster {
    /// Computes SFL chunk rates for each same-block dependency component.
    #[allow(clippy::too_many_arguments)]
    fn compute(
        &mut self,
        txin_starts: &[TxInIndex],
        outpoints: &[OutPoint],
        outpoint_base: usize,
        first_tx: usize,
        fees: &[Sats],
        vsizes: &[VSize],
        fee_rates: &[FeeRate],
        effective_fee_rates: &mut Vec<FeeRate>,
    ) {
        let n = fees.len();
        debug_assert_eq!(vsizes.len(), n);
        debug_assert_eq!(fee_rates.len(), n);
        debug_assert_eq!(txin_starts.len(), n);

        effective_fee_rates.clear();
        effective_fee_rates.extend_from_slice(fee_rates);
        self.parents.clear();
        self.parents.resize_with(n, SmallVec::new);
        self.roots.clear();
        self.roots.extend(0..n);
        self.members.clear();
        self.local_index.clear();
        self.local_index.resize(n, usize::MAX);

        for child in 0..n {
            let mut parents: SmallVec<[usize; 2]> =
                Self::same_block_parents(child, txin_starts, outpoints, outpoint_base, first_tx, n)
                    .collect();
            parents.sort_unstable();
            parents.dedup();
            for &parent in &parents {
                Self::union(&mut self.roots, child, parent);
            }
            self.parents[child] = parents;
        }

        for tx in 0..n {
            self.members.push((Self::root(&mut self.roots, tx), tx));
        }
        self.members.sort_unstable();

        let mut start = 0;
        while start < n {
            let component_root = self.members[start].0;
            let end = self.members[start..]
                .partition_point(|&(candidate, _)| candidate == component_root)
                + start;
            if end - start > 1 {
                Self::linearize_component(
                    &self.members[start..end],
                    &self.parents,
                    fees,
                    vsizes,
                    effective_fee_rates,
                    &mut self.local_index,
                    &mut self.local_parents,
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
        let left = Self::root(roots, left);
        let right = Self::root(roots, right);
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
}

#[cfg(test)]
mod tests {
    use super::Cluster;
    use brk_types::{FeeRate, OutPoint, Sats, TxInIndex, TxIndex, VSize, Vout};

    #[test]
    fn marks_actual_cpfp_roles() {
        let mut rates = Vec::new();
        Cluster::default().compute(
            &[TxInIndex::from(0usize), TxInIndex::from(1usize)],
            &[
                OutPoint::COINBASE,
                OutPoint::new(TxIndex::from(10usize), Vout::ZERO),
            ],
            0,
            10,
            &[Sats::new(100), Sats::new(200)],
            &[VSize::new(100), VSize::new(100)],
            &[FeeRate::new(1.0), FeeRate::new(2.0)],
            &mut rates,
        );

        assert_eq!(rates, [FeeRate::new(1.5), FeeRate::new(1.5)]);
        assert_eq!(
            [
                (rates[0] > FeeRate::new(1.0), rates[0] < FeeRate::new(1.0)),
                (rates[1] > FeeRate::new(2.0), rates[1] < FeeRate::new(2.0)),
            ],
            [(true, false), (false, true)]
        );
    }

    #[test]
    fn keeps_independent_transaction_rates_separate() {
        let mut rates = Vec::new();
        Cluster::default().compute(
            &[TxInIndex::from(0usize), TxInIndex::from(1usize)],
            &[
                OutPoint::COINBASE,
                OutPoint::new(TxIndex::from(9usize), Vout::ZERO),
            ],
            0,
            10,
            &[Sats::new(100), Sats::new(300)],
            &[VSize::new(100), VSize::new(100)],
            &[FeeRate::new(1.0), FeeRate::new(3.0)],
            &mut rates,
        );

        assert_eq!(rates, [FeeRate::new(1.0), FeeRate::new(3.0)]);
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
        let mut rates = Vec::new();

        cluster.compute(
            &txin_starts,
            &outpoints,
            0,
            10,
            &[Sats::ZERO, Sats::ZERO, Sats::new(3_000)],
            &vsizes,
            &[FeeRate::ZERO, FeeRate::ZERO, FeeRate::new(30.0)],
            &mut rates,
        );
        assert_eq!(
            rates,
            [FeeRate::new(15.0), FeeRate::new(0.0), FeeRate::new(15.0)]
        );

        cluster.compute(
            &txin_starts,
            &outpoints,
            0,
            10,
            &[Sats::ZERO, Sats::new(3_000), Sats::ZERO],
            &vsizes,
            &[FeeRate::ZERO, FeeRate::new(30.0), FeeRate::ZERO],
            &mut rates,
        );
        assert_eq!(
            rates,
            [FeeRate::new(15.0), FeeRate::new(15.0), FeeRate::new(0.0)]
        );
    }
}
