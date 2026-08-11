use brk_traversable::Traversable;

#[derive(Traversable)]
pub struct WeightedModes<T> {
    pub cointime: T,
    pub coinflow: T,
    pub coinflow_8y: T,
    pub coinflow_4y: T,
    pub coinflow_2y: T,
    pub coinflow_1y: T,
    pub coinflow_6m: T,
    pub coinflow_3m: T,
    pub coinflow_1m: T,
}
