use brk_traversable::Traversable;

#[derive(Clone, Traversable)]
pub struct Mobility<T> {
    pub mobile: T,
    pub immobile: T,
}
