#[cfg(feature = "storage")]
pub use vecdb::CheckedSub;

/// Checked subtraction for domain values; storage builds use `vecdb`'s identical trait instead.
#[cfg(not(feature = "storage"))]
pub trait CheckedSub<Rhs = Self>: Sized {
    fn checked_sub(self, rhs: Rhs) -> Option<Self>;
}

#[cfg(not(feature = "storage"))]
macro_rules! impl_checked_sub {
    ($($t:ty)*) => {$(
        impl CheckedSub for $t {
            #[inline]
            fn checked_sub(self, rhs: Self) -> Option<Self> {
                <$t>::checked_sub(self, rhs)
            }
        }
    )*};
}

#[cfg(not(feature = "storage"))]
impl_checked_sub! { i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize }
