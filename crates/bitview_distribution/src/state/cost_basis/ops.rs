use brk_types::{Cents, Sats};

/// What a cohort keeps of its coins by creation price beyond its realized totals: nothing, or
/// its split at the spot ([`super::SpotSplit`]).
pub trait CostBasisOps: Send + Sync + 'static {
    fn increment(&mut self, price: Cents, sats: Sats);
    fn decrement(&mut self, price: Cents, sats: Sats);
}

// UTXO amount/type cohorts need realized accounting but no split.
impl CostBasisOps for () {
    fn increment(&mut self, _price: Cents, _sats: Sats) {}
    fn decrement(&mut self, _price: Cents, _sats: Sats) {}
}
