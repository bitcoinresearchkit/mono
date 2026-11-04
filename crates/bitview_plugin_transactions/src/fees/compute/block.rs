use bitview_plugin_indexer::FIRST_V29_POLICY_HEIGHT;
use bitview_primitives::TxInIndex;
use brk_cpfp::Cluster;
use brk_types::{CpfpClusterTxIndex, FeeRate, OutPoint, Sats, VSize};
use smallvec::SmallVec;
use vecdb::VecIndex;

use super::kinds::{
    BITS, CPFP_CHILD, CPFP_PARENT, Columns, DUST, INSCRIPTION, NONSTANDARD, READ, UNCONDITIONAL,
};
use crate::distribution::{Stats, counted, weighted};

#[derive(Default)]
pub struct Block {
    first_tx: usize,
    /// Its height, and its first transaction's place in the batch, for the batch's flags.
    pub height: usize,
    pub batch_offset: usize,
    /// A bit for each kind each transaction is (see `kinds`): those derived from the reads at
    /// first, every other once computed.
    pub masks: Vec<u64>,
    /// The value of each kind's transactions (`masks`' bits), the coinbase's none.
    pub values: Vec<Sats>,
    pub inscription_fees: Sats,
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
    /// Distribution statistics of the non-coinbase fees, the non-coinbase effective rates
    /// weighted by vsize, and every transaction's vsize.
    pub fee_stats: Stats<Sats>,
    pub rate_stats: Stats<FeeRate>,
    pub vsize_stats: Stats<VSize>,
    sorted_fees: Vec<Sats>,
    weighted_rates: Vec<(FeeRate, VSize)>,
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
        self.masks.clear();
    }

    /// `components`: scratch reused across blocks, one per worker. `bits`: the batch's flags.
    pub fn compute(&mut self, components: &mut Components, bits: &[Vec<u64>; READ]) {
        let n = self.input_values.len();
        debug_assert_eq!(n, self.output_values.len());
        debug_assert_eq!(n, self.vsizes.len());
        debug_assert_eq!(n, self.txin_starts.len());
        debug_assert_eq!(n, self.masks.len());
        // The coinbase spends nothing: its single input carries the `Sats::MAX` sentinel.
        debug_assert!(self.input_values[0].is_max());

        self.fees.reserve(n);
        self.fee_rates.reserve(n);
        self.fees.push(Sats::ZERO);
        self.fee_rates
            .push(FeeRate::from((Sats::ZERO, self.vsizes[0])));
        for ((&input, &output), &vsize) in self.input_values[1..]
            .iter()
            .zip(&self.output_values[1..])
            .zip(&self.vsizes[1..])
        {
            let fee = input - output;
            self.transfer_volume += input;
            self.total_fee += fee;
            self.fees.push(fee);
            self.fee_rates.push(FeeRate::from((fee, vsize)));
        }
        components.compute(
            &self.txin_starts,
            &self.outpoints,
            self.input_begin,
            self.first_tx,
            &self.fees,
            &self.vsizes,
            &self.fee_rates,
            &mut self.effective_fee_rates,
        );

        self.value_kinds(bits);

        self.sorted_fees.clear();
        self.sorted_fees.extend_from_slice(&self.fees[1..]);
        self.fee_stats = counted(&mut self.sorted_fees);
        self.weighted_rates.clear();
        self.weighted_rates.extend(
            self.effective_fee_rates[1..]
                .iter()
                .copied()
                .zip(self.vsizes[1..].iter().copied()),
        );
        self.rate_stats = weighted(&mut self.weighted_rates);
        // Last use of the block-order vsizes: sorted in place.
        self.vsize_stats = counted(&mut self.vsizes);
    }

    /// Completes each transaction's mask (its flags, CPFP role and nonstandard) and sums the
    /// value of each kind's transactions, and inscriptions' fees.
    fn value_kinds(&mut self, bits: &[Vec<u64>; READ]) {
        self.values.clear();
        self.values.resize(BITS, Sats::ZERO);
        self.inscription_fees = Sats::ZERO;
        // Bitcoin Core 29 relays dust when it pays no fee (ephemeral dust).
        let free_dust_relayed = self.height >= FIRST_V29_POLICY_HEIGHT as usize;
        Columns::or_into(bits, self.batch_offset, &mut self.masks);
        for tx in 0..self.masks.len() {
            let fee = self.fees[tx];
            let (raw, effective) = (self.fee_rates[tx], self.effective_fee_rates[tx]);
            let mut mask = self.masks[tx];
            mask |= (u64::from(effective > raw) << CPFP_PARENT)
                | (u64::from(effective < raw) << CPFP_CHILD);
            let nonstandard = is(mask, UNCONDITIONAL)
                || (is(mask, DUST) && (!free_dust_relayed || fee != Sats::ZERO));
            mask |= u64::from(nonstandard) << NONSTANDARD;
            self.masks[tx] = mask;
            if is(mask, INSCRIPTION) {
                self.inscription_fees += fee;
            }
            // The coinbase spends nothing.
            if tx > 0 {
                let value = self.input_values[tx];
                let mut rest = mask;
                while rest != 0 {
                    self.values[rest.trailing_zeros() as usize] += value;
                    rest &= rest - 1;
                }
            }
        }
    }
}

/// Whether `mask` has `bit`.
#[inline(always)]
pub fn is(mask: u64, bit: usize) -> bool {
    mask >> bit & 1 == 1
}

/// A block's same-block dependency components, linearized one at a time.
#[derive(Default)]
pub struct Components {
    parents: Vec<SmallVec<[usize; 2]>>,
    roots: Vec<usize>,
    members: Vec<(usize, usize)>,
    local_index: Vec<u32>,
    cluster: Cluster,
}

impl Components {
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
                &mut self.cluster,
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
        cluster: &mut Cluster,
    ) {
        for (local, &(_, tx)) in members.iter().enumerate() {
            local_index[tx] = local as u32;
        }

        cluster.clear();
        for &(_, tx) in members {
            cluster.push(
                fees[tx],
                vsizes[tx],
                parents[tx]
                    .iter()
                    .map(|&parent| CpfpClusterTxIndex::from(local_index[parent])),
            );
        }

        cluster.linearize(|txs, feerate| {
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
