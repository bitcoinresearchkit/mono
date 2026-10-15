use super::UnaryTransform;

/// v -> v.into(): a conversion that is just `From`/`Into` (e.g. a lossless widening).
pub struct Convert;

impl<A: Into<B>, B> UnaryTransform<A, B> for Convert {
    #[inline(always)]
    fn apply(value: A) -> B {
        value.into()
    }
}
