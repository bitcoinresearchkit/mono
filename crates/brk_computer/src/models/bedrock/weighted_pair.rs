use brk_types::UrpdWeight;

#[derive(Default)]
pub(super) struct WeightedPair<T> {
    pub(super) cointime: T,
    pub(super) coinflow: T,
}

impl<T> WeightedPair<T> {
    pub(super) fn from_fn(mut create: impl FnMut(UrpdWeight) -> T) -> Self {
        Self {
            cointime: create(UrpdWeight::Cointime),
            coinflow: create(UrpdWeight::Coinflow),
        }
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = &T> {
        [&self.cointime, &self.coinflow].into_iter()
    }
}
