use brk_oracle::{HistogramRaw, PaymentFilter, sats_to_bin};
use brk_types::Transaction;

use crate::stores::tx_store::TxRecord;

/// The two live per-bin histograms the pool maintains incrementally as txs
/// enter and leave: `eligible` applies the round-dollar payment filter (it
/// feeds the oracle blend), `raw` bins outputs by value with no filtering.
/// Add and remove run through the same code so the two stay symmetric.
#[derive(Clone, Default)]
pub struct LiveHistograms {
    eligible: HistogramRaw,
    raw: HistogramRaw,
}

impl LiveHistograms {
    pub fn add(&mut self, record: &TxRecord) {
        Self::eligible_bins(&record.tx, |bin| self.eligible[bin as usize] += 1);
        for bin in Self::raw_bins(&record.tx) {
            self.raw[bin] += 1;
        }
    }

    /// Reverse a previous `add` for the same record.
    pub fn remove(&mut self, record: &TxRecord) {
        Self::eligible_bins(&record.tx, |bin| self.eligible[bin as usize] -= 1);
        for bin in Self::raw_bins(&record.tx) {
            self.raw[bin] -= 1;
        }
    }

    /// Round-dollar-eligible bins, blended into the oracle by `live_price`.
    pub fn eligible(&self) -> HistogramRaw {
        self.eligible.clone()
    }

    /// Live outputs binned by value, no payment filtering.
    pub fn raw(&self) -> HistogramRaw {
        self.raw.clone()
    }

    /// Round-dollar-eligible bins, applying the oracle payment filter. Calls
    /// `emit(bin)` per eligible output. Deterministic over a tx's outputs,
    /// which are never mutated after insert, so add and remove recompute it
    /// identically rather than caching.
    fn eligible_bins(tx: &Transaction, emit: impl FnMut(u16)) {
        PaymentFilter::MODERN
            .for_each_bin(tx.output.iter().map(|o| (o.value, o.output_type())), emit);
    }

    /// Raw bin index per output, dropping only values outside the bin domain
    /// (zero / out-of-range).
    fn raw_bins(tx: &Transaction) -> impl Iterator<Item = usize> + '_ {
        tx.output.iter().filter_map(|o| sats_to_bin(o.value))
    }
}
