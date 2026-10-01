use brk_types::{Cents, Sats};

/// Optional price-distribution tracking. Realized totals belong to realized state.
pub trait CostBasisOps: Send + Sync + 'static {
    fn increment(&mut self, price: Cents, sats: Sats);
    fn decrement(&mut self, price: Cents, sats: Sats);
    fn apply_pending(&mut self);
    fn init(&mut self);
}

// UTXO amount/type cohorts need realized accounting but no price distribution.
impl CostBasisOps for () {
    fn increment(&mut self, _price: Cents, _sats: Sats) {}
    fn decrement(&mut self, _price: Cents, _sats: Sats) {}
    fn apply_pending(&mut self) {}
    fn init(&mut self) {}
}
