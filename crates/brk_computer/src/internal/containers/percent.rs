use brk_traversable::Traversable;

#[derive(Clone, Traversable)]
pub struct Percent<A, B = A, C = B> {
    /// Unitless ratio in parts per million; 1,000,000 represents 1.0.
    pub ppm: A,
    /// Unitless decimal ratio derived as parts per million divided by 1,000,000.
    pub ratio: B,
    /// Percentage derived as the decimal ratio multiplied by 100.
    pub percent: C,
}
