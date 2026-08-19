use brk_types::{
    PartsPerMillion32, PartsPerMillion64, PartsPerMillionSigned32, PartsPerMillionSigned64,
    StoredF32,
};
use vecdb::UnaryTransform;

pub struct FixedToRatio;
pub struct FixedToPercent;

macro_rules! impl_fixed_ratio {
    ($($type:ty),+ $(,)?) => {
        $(
            impl UnaryTransform<$type, StoredF32> for FixedToRatio {
                #[inline(always)]
                fn apply(value: $type) -> StoredF32 {
                    StoredF32::from(value.to_f32())
                }
            }

            impl UnaryTransform<$type, StoredF32> for FixedToPercent {
                #[inline(always)]
                fn apply(value: $type) -> StoredF32 {
                    StoredF32::from(value.to_f32() * 100.0)
                }
            }
        )+
    };
}

impl_fixed_ratio!(
    PartsPerMillion32,
    PartsPerMillionSigned32,
    PartsPerMillion64,
    PartsPerMillionSigned64,
);
