use bitview_traversable::Traversable;

#[derive(Debug, Default, Traversable)]
pub struct ByAnyAddr<T> {
    pub funded: T,
    pub empty: T,
}

impl<T> ByAnyAddr<Option<T>> {
    pub fn take(&mut self) {
        self.funded.take();
        self.empty.take();
    }
}
