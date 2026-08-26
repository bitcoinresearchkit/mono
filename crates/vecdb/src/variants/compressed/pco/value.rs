use crate::{BytesVecValue, Pco};

/// Marker trait for values storable in a `PcoVec`: must be `Copy`, `Pco`,
/// and serializable via the `Bytes` path used by `BytesVec`.
pub trait PcoVecValue: Pco + BytesVecValue + Copy {}

impl<T> PcoVecValue for T where T: Pco + BytesVecValue + Copy {}

macro_rules! impl_pco_primitive {
    ($($t:ty),*) => {
        $(
            // SAFETY: The value and its PCO number are the same primitive type.
            unsafe impl Pco for $t {
                type NumberType = $t;
                const IS_TRANSPARENT: bool = true;

                #[inline(always)]
                fn to_number(self) -> Self::NumberType {
                    self
                }

                #[inline(always)]
                fn from_number(value: Self::NumberType) -> crate::Result<Self> {
                    Ok(value)
                }
            }
        )*
    };
}

impl_pco_primitive!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);
