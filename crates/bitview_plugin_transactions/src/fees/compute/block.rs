use bitview_primitives::TxInIndex;
use brk_cpfp::Cluster as Graph;
use brk_types::{CpfpClusterTxIndex, FeeRate, OutPoint, Sats, VSize};
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

    /// `cluster`: scratch reused across blocks, one per worker.
    pub fn compute(&mut self, cluster: &mut Cluster) {
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
        self.compute_cluster(cluster);
    }

    /// Repair rates/CPFP from valid fees without decoding monetary inputs again.
    pub fn compute_from_fees(&mut self, cluster: &mut Cluster) {
        self.fee_rates.clear();
        self.fee_rates.extend(
            self.fees
                .iter()
                .zip(&self.vsizes)
                .map(|(&fee, &size)| FeeRate::from((fee, size))),
        );
        self.compute_cluster(cluster);
    }

    fn compute_cluster(&mut self, cluster: &mut Cluster) {
        cluster.compute(
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
pub struct Cluster {
    parents: Vec<SmallVec<[usize; 2]>>,
    roots: Vec<usize>,
    members: Vec<(usize, usize)>,
    local_index: Vec<u32>,
    graph: Graph,
}

impl Cluster {
    /// Computes chunk rates for each same-block dependency component.
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
                &mut self.graph,
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
        local_index: &mut [u32],
        graph: &mut Graph,
    ) {
        for (local, &(_, tx)) in members.iter().enumerate() {
            local_index[tx] = local as u32;
        }

        graph.clear();
        for &(_, tx) in members {
            graph.push(
                fees[tx],
                vsizes[tx],
                parents[tx]
                    .iter()
                    .map(|&parent| CpfpClusterTxIndex::from(local_index[parent])),
            );
        }

        graph.linearize(|txs, feerate| {
            for &local in txs {
                rates[members[u32::from(local) as usize].1] = feerate;
            }
        });
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
