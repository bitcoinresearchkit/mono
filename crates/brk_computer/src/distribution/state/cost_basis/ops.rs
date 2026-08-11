use std::path::Path;

use brk_error::Result;
use brk_types::{Cents, CentsSats, CentsSquaredSats, Height, Sats};

/// Common interface for persisted cost-basis state.
pub trait CostBasisOps: Send + Sync + 'static {
    fn create(path: &Path, name: &str) -> Self;
    fn import_at_or_before(&mut self, height: Height) -> Result<Height>;
    fn cap_raw(&self) -> CentsSats;
    fn capitalized_cap_raw(&self) -> CentsSquaredSats;
    fn increment(
        &mut self,
        price: Cents,
        sats: Sats,
        price_sats: CentsSats,
        capitalized_cap: CentsSquaredSats,
    );
    fn decrement(
        &mut self,
        price: Cents,
        sats: Sats,
        price_sats: CentsSats,
        capitalized_cap: CentsSquaredSats,
    );
    fn apply_pending(&mut self);
    fn init(&mut self);
    fn clean(&mut self) -> Result<()>;
    fn write(&mut self, height: Height, cleanup: bool) -> Result<()>;
}
