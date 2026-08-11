use brk_types::{Cents, CentsSats, CentsSquaredSats, Sats};

/// Trait for realized state operations, implemented by Minimal, Core, and Full variants.
pub trait RealizedOps: Default + Clone + Send + Sync + 'static {
    const TRACK_ACTIVITY: bool = false;
    fn cap_raw(&self) -> CentsSats;
    fn cap(&self) -> Cents;
    fn profit(&self) -> Cents;
    fn loss(&self) -> Cents;
    fn value_destroyed(&self) -> Cents {
        Cents::ZERO
    }
    fn sent_in_profit(&self) -> Sats {
        Sats::ZERO
    }
    fn sent_in_loss(&self) -> Sats {
        Sats::ZERO
    }
    fn set_cap_raw(&mut self, cap_raw: CentsSats);
    fn set_capitalized_cap_raw(&mut self, capitalized_cap_raw: CentsSquaredSats);
    fn reset_single_iteration_values(&mut self);
    fn increment(&mut self, price: Cents, sats: Sats);
    fn increment_snapshot(&mut self, price_sats: CentsSats, capitalized_cap: CentsSquaredSats);
    fn decrement_snapshot(&mut self, price_sats: CentsSats, capitalized_cap: CentsSquaredSats);
    fn receive(&mut self, price: Cents, sats: Sats) {
        self.increment(price, sats);
    }
    fn send(
        &mut self,
        sats: Sats,
        current_ps: CentsSats,
        prev_ps: CentsSats,
        ath_ps: CentsSats,
        prev_capitalized_cap: CentsSquaredSats,
    );
}
