use bitview_traversable::Traversable;
use brk_types::UrpdWeight;

#[derive(Default, Traversable)]
pub struct WeightedPair<T> {
    pub cointime: T,
    pub coinflow: T,
}

impl<T> WeightedPair<T> {
    pub fn from_fn(mut create: impl FnMut(UrpdWeight) -> T) -> Self {
        Self {
            cointime: create(UrpdWeight::Cointime),
            coinflow: create(UrpdWeight::Coinflow),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [&self.cointime, &self.coinflow].into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.cointime, &mut self.coinflow].into_iter()
    }
}
