use vecdb::UnaryTransform;

/// A value stored or computed at double precision as its published type (`Ratio64` as `Ratio`,
/// `Float64` as `Float`): clients never see a storage size.
pub struct Narrow;

impl<S: Into<f64>, O: From<f64>> UnaryTransform<S, O> for Narrow {
    #[inline(always)]
    fn apply(value: S) -> O {
        O::from(value.into())
    }
}
