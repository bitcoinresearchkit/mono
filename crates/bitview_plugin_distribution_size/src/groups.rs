use bitview_cohort::{AmountRange, CohortId, SpendableType};
use bitview_traversable::Traversable;
#[derive(Clone, Traversable)]
pub struct SizeGroups<T> {
    pub utxo_amount: AmountRange<T>,
    pub type_: SpendableType<T>,
}
impl<T> SizeGroups<T> {
    pub fn new(mut f: impl FnMut(CohortId) -> T) -> Self {
        Self {
            utxo_amount: AmountRange::new(&mut f),
            type_: SpendableType::new(f),
        }
    }
    pub fn try_new<E>(mut f: impl FnMut(CohortId) -> Result<T, E>) -> Result<Self, E> {
        Ok(Self {
            utxo_amount: AmountRange::try_new(&mut f)?,
            type_: SpendableType::try_new(f)?,
        })
    }
    pub fn get(&self, id: CohortId) -> Option<&T> {
        match id {
            CohortId::Amount(id) => Some(id.select(&self.utxo_amount)),
            CohortId::Type(id) => Some(self.type_.get(id)),
            _ => None,
        }
    }
    pub fn map_with_id<U: Clone>(&self, mut f: impl FnMut(CohortId, &T) -> U) -> SizeGroups<U> {
        SizeGroups::new(|id| f(id, self.get(id).expect("size cohort")))
    }
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.utxo_amount.iter().chain(self.type_.iter())
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.utxo_amount.iter_mut().chain(self.type_.iter_mut())
    }
}
