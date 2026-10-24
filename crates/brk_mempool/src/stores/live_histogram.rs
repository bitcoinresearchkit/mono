use brk_oracle::{HistogramRaw, PaymentFilter};
use brk_types::Transaction;

use crate::stores::tx_store::TxRecord;

/// The live per-bin histogram of round-dollar-eligible outputs the pool maintains incrementally as
/// txs enter and leave (it feeds the oracle blend). Add and remove run through the same code so
/// they stay symmetric.
#[derive(Clone, Default)]
pub struct LiveHistogram(HistogramRaw);

impl LiveHistogram {
    pub fn add(&mut self, record: &TxRecord) {
        Self::eligible_bins(&record.tx, |bin| self.0[bin as usize] += 1);
    }

    /// Reverse a previous `add` for the same record.
    pub fn remove(&mut self, record: &TxRecord) {
        Self::eligible_bins(&record.tx, |bin| self.0[bin as usize] -= 1);
    }

    /// Round-dollar-eligible bins, blended into the oracle by `live_price`.
    pub fn eligible(&self) -> HistogramRaw {
        self.0.clone()
    }

    /// Round-dollar-eligible bins, applying the oracle payment filter. Calls
    /// `emit(bin)` per eligible output. Deterministic over a tx's outputs,
    /// which are never mutated after insert, so add and remove recompute it
    /// identically rather than caching.
    fn eligible_bins(tx: &Transaction, emit: impl FnMut(u16)) {
        PaymentFilter::MODERN
            .for_each_bin(tx.output.iter().map(|o| (o.value, o.output_type())), emit);
    }
}
