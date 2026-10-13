use brk_types::{CentsSigned, Dollars};
use vecdb::BinaryTransform;

/// AVIV NUPL: active capitalization above investor capitalization, as a share of active
/// capitalization (one minus one over AVIV, from the capitalizations themselves). Undefined while
/// active capitalization is zero.
pub struct AvivNupl;

impl<P: From<f64>> BinaryTransform<Dollars, CentsSigned, P> for AvivNupl {
    #[inline(always)]
    fn apply(active_cap: Dollars, investor_cap: CentsSigned) -> P {
        let active_cents = f64::from(active_cap) * 100.0;
        P::from((active_cents - f64::from(investor_cap)) / active_cents)
    }
}
