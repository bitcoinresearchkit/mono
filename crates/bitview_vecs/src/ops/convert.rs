use vecdb::UnaryTransform;

/// A conversion that is just `From`/`Into` (e.g. sats to bitcoin, cents to dollars).
pub struct Convert;

impl<A, B> UnaryTransform<A, B> for Convert
where
    A: Into<B>,
{
    #[inline(always)]
    fn apply(value: A) -> B {
        value.into()
    }
}
