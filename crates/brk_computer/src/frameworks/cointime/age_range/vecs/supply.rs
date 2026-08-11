use brk_traversable::Traversable;

#[derive(Traversable)]
pub struct SupplyVecs<T> {
    pub awake: T,
    pub dormant: T,
}
