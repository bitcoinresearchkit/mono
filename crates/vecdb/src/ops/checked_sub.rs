/// Subtraction that returns `None` on overflow. Implemented for the primitive integers.
pub trait CheckedSub<Rhs = Self>: Sized {
    fn checked_sub(self, rhs: Rhs) -> Option<Self>;
}

macro_rules! impl_checked_sub {
    ($($t:ty)*) => ($(
        impl CheckedSub for $t {
            #[inline]
            fn checked_sub(self, rhs: Self) -> Option<Self> {
                <$t>::checked_sub(self, rhs)
            }
        }
    )*)
}

impl_checked_sub! { i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize }
