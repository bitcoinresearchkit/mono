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
    pub transfer_volume: Sats,
    pub total_fee: Sats,
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
        self.transfer_volume = Sats::ZERO;
        self.total_fee = Sats::ZERO;
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
                self.transfer_volume += input;
                input - output
            };
            self.total_fee += fee;
            self.fees.push(fee);
            self.fee_rates.push(FeeRate::from((fee, vsize)));
        }
        self.compute_cluster();
    }

    /// Repair rates/CPFP from valid fees without decoding monetary inputs again.
    pub fn compute_from_fees(&mut self) {
        self.fee_rates.clear();
        self.fee_rates.extend(
            self.fees
                .iter()
                .zip(&self.vsizes)
                .map(|(&fee, &size)| FeeRate::from((fee, size))),
        );
        self.compute_cluster();
    }

    fn compute_cluster(&mut self) {
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

        // Union attaches parent roots to children, so connected roots have parents.
        // Transactions with neither parents nor children keep their raw rate.
        for tx in 0..n {
            if !self.parents[tx].is_empty() || self.roots[tx] != tx {
                self.members.push((Self::root(&mut self.roots, tx), tx));
            }
        }
        self.local_index.resize(n, 0);
        self.members.sort_unstable();

        let mut start = 0;
        while start < self.members.len() {
            let component_root = self.members[start].0;
            let end = self.members[start..]
                .partition_point(|&(candidate, _)| candidate == component_root)
                + start;
            Self::linearize_component(
                &self.members[start..end],
                &self.parents,
                fees,
                vsizes,
                effective_fee_rates,
                &mut self.local_index,
                &mut self.local_parents,
            );
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
    fn keeps_independent_transaction_rates_separate() {
        let mut rates = Vec::new();
        // Reuse the scratch state across changing block sizes, mixing isolated
        // transactions with a chain whose first child spends its parent twice.
        let mut cluster = Cluster::default();
        for n in [7, 3, 9, 1, 0, 5, 10, 2] {
            let mut starts = Vec::new();
            let mut outpoints = Vec::new();
            let mut fees = Vec::new();
            for tx in 0..n {
                starts.push(TxInIndex::from(outpoints.len()));
                match tx {
                    0 => outpoints.push(OutPoint::COINBASE),
                    2 => outpoints.extend([
                        OutPoint::new(TxIndex::from(10usize), Vout::ZERO),
                        OutPoint::new(TxIndex::from(10usize), Vout::from(1u32)),
                    ]),
                    4 => outpoints.push(OutPoint::new(TxIndex::from(12usize), Vout::ZERO)),
                    _ => outpoints.push(OutPoint::new(TxIndex::from(9usize), Vout::ZERO)),
                }
                fees.push(Sats::from(match tx {
                    0 => 0u64,
                    2 => 200,
                    4 => 100,
                    _ => (tx as u64 + 1) * 100,
                }));
            }
            let sizes = vec![VSize::new(100); n];
            let raw: Vec<_> = fees
                .iter()
                .map(|&fee| FeeRate::from((fee, sizes[0])))
                .collect();
            let mut expected = raw.clone();
            if n >= 3 {
                for tx in [0, 2, 4].into_iter().filter(|&tx| tx < n) {
                    expected[tx] = FeeRate::new(1.0);
                }
            }
            cluster.compute(&starts, &outpoints, 0, 10, &fees, &sizes, &raw, &mut rates);
            assert_eq!(rates, expected, "block length {n}");
        }
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
    #[test]
    fn rate_repair_reuses_the_same_fees_and_cpfp_results() {
        let mut block = super::Block::default();
        block.reset(10);
        block.input_values = vec![Sats::MAX, Sats::new(2_000), Sats::new(3_000)];
        block.output_values = vec![Sats::new(5_000), Sats::new(1_900), Sats::new(2_800)];
        block.vsizes = vec![VSize::new(100); 3];
        block.txin_starts = [0usize, 1, 2].map(TxInIndex::from).into();
        block.outpoints = vec![
            OutPoint::COINBASE,
            OutPoint::COINBASE,
            OutPoint::new(TxIndex::from(11usize), Vout::ZERO),
        ];
        block.compute();
        assert_eq!(block.total_fee, Sats::new(300));
        assert_eq!(block.transfer_volume, Sats::new(5_000));
        let fees = block.fees.clone();
        let rates = block.fee_rates.clone();
        let effective = block.effective_fee_rates.clone();
        block.input_values.clear();
        block.output_values.clear();
        block.fee_rates.clear();
        block.effective_fee_rates.clear();
        block.compute_from_fees();
        assert_eq!(block.fees, fees);
        assert_eq!(block.fee_rates, rates);
        assert_eq!(block.effective_fee_rates, effective);
    }
}
