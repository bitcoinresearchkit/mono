use super::UnaryTransform;

pub struct Convert;

impl<A: Into<B>, B> UnaryTransform<A, B> for Convert {
    #[inline(always)]
    fn apply(value: A) -> B {
        value.into()
    }
}
