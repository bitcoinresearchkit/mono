use brk_oracle::{HistogramRaw, PaymentFilter, Tally};
use brk_types::{OutputType, Sats, Transaction};

use crate::stores::tx_store::TxRecord;

/// What the pool's live outputs make of the oracle's filter, maintained incrementally as txs enter
/// and leave: the per-bin histogram of round-dollar-eligible outputs (it feeds the oracle blend),
/// and every output under the rule that skips it. Add and remove run through the same code so
/// they stay symmetric.
#[derive(Clone, Default)]
pub struct LivePayments {
    histogram: HistogramRaw,
    tally: Tally,
}

impl LivePayments {
    pub fn add(&mut self, record: &TxRecord) {
        let outputs = Self::outputs(&record.tx);
        PaymentFilter::MODERN
            .for_each_bin(outputs.clone(), |bin| self.histogram[bin as usize] += 1);
        PaymentFilter::MODERN.classify(outputs, |skip, n| self.tally.add(skip, n));
    }

    /// Reverse a previous `add` for the same record.
    pub fn remove(&mut self, record: &TxRecord) {
        let outputs = Self::outputs(&record.tx);
        PaymentFilter::MODERN
            .for_each_bin(outputs.clone(), |bin| self.histogram[bin as usize] -= 1);
        PaymentFilter::MODERN.classify(outputs, |skip, n| self.tally.remove(skip, n));
    }

    /// Round-dollar-eligible bins, blended into the oracle by `live_price`, and the outputs'
    /// tally, together.
    pub fn payments(&self) -> (HistogramRaw, Tally) {
        (self.histogram.clone(), self.tally)
    }

    /// A tx's outputs as the filter reads them. Deterministic over a tx's outputs, which are
    /// never mutated after insert, so add and remove recompute it identically rather than caching.
    fn outputs(tx: &Transaction) -> impl ExactSizeIterator<Item = (Sats, OutputType)> + Clone + '_ {
        tx.output.iter().map(|o| (o.value, o.output_type()))
    }
}
