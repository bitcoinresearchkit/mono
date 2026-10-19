/// Addition that clamps to the type's min/max on overflow instead of wrapping. Implemented for
/// the primitive integers.
pub trait SaturatingAdd<Rhs = Self>: Sized {
    fn saturating_add(self, rhs: Rhs) -> Self;
}

macro_rules! impl_saturating_add {
    ($($t:ty)*) => ($(
        impl SaturatingAdd for $t {
            #[inline]
            fn saturating_add(self, rhs: Self) -> Self {
                <$t>::saturating_add(self, rhs)
            }
        }
    )*)
}

impl_saturating_add! { i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize }
