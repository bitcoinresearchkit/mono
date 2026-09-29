use super::UTXOStates;
use bitview_cohort::AgeRangeId;
use bitview_urpd::COST_BASIS_PRICE_DIGITS;
use brk_types::{CentsCompact, Sats};
impl UTXOStates {
    /// Bounds need only each age range's first and last occupied price.
    pub fn bounds_entries(&self) -> impl Iterator<Item = (AgeRangeId, CentsCompact, Sats)> + '_ {
        AgeRangeId::ALL.iter().copied().flat_map(|age| {
            let map = age.select(&self.age_range).cost_basis_map();
            map.first_key_value()
                .into_iter()
                .chain(map.last_key_value())
                .map(move |(&price, &sats)| {
                    (age, price.round_to_dollar(COST_BASIS_PRICE_DIGITS), sats)
                })
        })
    }
}
