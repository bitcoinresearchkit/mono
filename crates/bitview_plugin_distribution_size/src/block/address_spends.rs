use bitview_cohort::ByAddrType;
use brk_types::{Cents, Height, OutputType, Sats, TypeIndex};
use rustc_hash::FxHashMap;
/// Keeps the established creation-height group order and input order within each group.
pub struct AddressSpends(FxHashMap<Height, Vec<(OutputType, TypeIndex, Sats)>>);
impl AddressSpends {
    pub fn new(capacity: usize) -> Self {
        Self(FxHashMap::with_capacity_and_hasher(
            capacity,
            Default::default(),
        ))
    }
    pub fn push(&mut self, height: Height, ty: OutputType, index: TypeIndex, value: Sats) {
        self.0.entry(height).or_default().push((ty, index, value));
    }
    pub fn into_typed(self, prices: &[Cents]) -> ByAddrType<Vec<(TypeIndex, Sats, Cents)>> {
        let mut typed = ByAddrType::<Vec<(TypeIndex, Sats, Cents)>>::default();
        for (height, spends) in self.0 {
            let price = prices[usize::from(height)];
            for (ty, index, value) in spends {
                typed.get_mut_unwrap(ty).push((index, value, price));
            }
        }
        typed
    }
}
