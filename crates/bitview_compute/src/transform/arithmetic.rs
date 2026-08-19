use std::marker::PhantomData;

use brk_types::{
    Bitcoin, Cents, Dollars, PartsPerMillion32, Sats, StoredF32, StoredF64, StoredI8, StoredU16,
    StoredU32, StoredU64, VSize, Weight, Weight64,
};
use vecdb::{BinaryTransform, UnaryTransform, VecValue};

pub struct Identity<T>(PhantomData<T>);

impl<T: VecValue> UnaryTransform<T, T> for Identity<T> {
    #[inline(always)]
    fn apply(v: T) -> T {
        v
    }
}

pub struct HalveSats;

impl UnaryTransform<Sats, Sats> for HalveSats {
    #[inline(always)]
    fn apply(sats: Sats) -> Sats {
        sats / 2
    }
}

pub struct HalveSatsToBitcoin;

impl UnaryTransform<Sats, Bitcoin> for HalveSatsToBitcoin {
    #[inline(always)]
    fn apply(sats: Sats) -> Bitcoin {
        Bitcoin::from(sats / 2)
    }
}

pub struct HalveCents;

impl UnaryTransform<Cents, Cents> for HalveCents {
    #[inline(always)]
    fn apply(cents: Cents) -> Cents {
        cents / 2u64
    }
}

pub struct HalveDollars;

impl UnaryTransform<Dollars, Dollars> for HalveDollars {
    #[inline(always)]
    fn apply(dollars: Dollars) -> Dollars {
        dollars.halved()
    }
}

pub struct MaskSats;

impl BinaryTransform<StoredU32, Sats, Sats> for MaskSats {
    #[inline(always)]
    fn apply(mask: StoredU32, value: Sats) -> Sats {
        if mask == StoredU32::ONE {
            value
        } else {
            Sats::ZERO
        }
    }
}

impl BinaryTransform<StoredU64, Sats, Sats> for MaskSats {
    #[inline]
    fn apply(mask: StoredU64, value: Sats) -> Sats {
        if u64::from(mask) != 0 {
            value
        } else {
            Sats::ZERO
        }
    }
}

pub struct ReturnF32Tenths<const V: u16>;

impl<S, const V: u16> UnaryTransform<S, StoredF32> for ReturnF32Tenths<V> {
    #[inline(always)]
    fn apply(_: S) -> StoredF32 {
        StoredF32::from(V as f32 / 10.0)
    }
}

pub struct ReturnU16<const V: u16>;

impl<S, const V: u16> UnaryTransform<S, StoredU16> for ReturnU16<V> {
    #[inline(always)]
    fn apply(_: S) -> StoredU16 {
        StoredU16::new(V)
    }
}

pub struct ReturnI8<const V: i8>;

impl<S, const V: i8> UnaryTransform<S, StoredI8> for ReturnI8<V> {
    #[inline(always)]
    fn apply(_: S) -> StoredI8 {
        StoredI8::new(V)
    }
}

pub struct ThsToPhsF32;

impl UnaryTransform<StoredF32, StoredF32> for ThsToPhsF32 {
    #[inline(always)]
    fn apply(ths: StoredF32) -> StoredF32 {
        (*ths * 1000.0).into()
    }
}

pub struct BlocksToDaysF32;

impl UnaryTransform<StoredU32, StoredF32> for BlocksToDaysF32 {
    #[inline(always)]
    fn apply(blocks: StoredU32) -> StoredF32 {
        (*blocks as f32 / crate::TARGET_BLOCKS_PER_DAY_F32).into()
    }
}

pub struct StoredU64ToStoredU32;

impl UnaryTransform<StoredU64, StoredU32> for StoredU64ToStoredU32 {
    #[inline(always)]
    fn apply(value: StoredU64) -> StoredU32 {
        let value = u64::from(value);
        debug_assert!(u32::try_from(value).is_ok());
        StoredU32::new(value as u32)
    }
}

pub struct StoredU16ToStoredU64;

impl UnaryTransform<StoredU16, StoredU64> for StoredU16ToStoredU64 {
    #[inline(always)]
    fn apply(value: StoredU16) -> StoredU64 {
        StoredU64::from(u64::from(*value))
    }
}

pub struct PerSecond<const SECONDS: u32>;

impl<const SECONDS: u32> UnaryTransform<StoredU64, StoredF32> for PerSecond<SECONDS> {
    #[inline(always)]
    fn apply(value: StoredU64) -> StoredF32 {
        StoredF32::from(u64::from(value) as f64 / SECONDS as f64)
    }
}

pub struct OneMinusF64;

impl UnaryTransform<StoredF64, StoredF64> for OneMinusF64 {
    #[inline(always)]
    fn apply(v: StoredF64) -> StoredF64 {
        StoredF64::from(1.0 - *v)
    }
}

pub struct OddsF64;

impl UnaryTransform<StoredF64, StoredF64> for OddsF64 {
    #[inline(always)]
    fn apply(value: StoredF64) -> StoredF64 {
        value / StoredF64::from(1.0 - *value)
    }
}

pub struct DifficultyToHashF64;

impl UnaryTransform<StoredF64, StoredF64> for DifficultyToHashF64 {
    #[inline(always)]
    fn apply(difficulty: StoredF64) -> StoredF64 {
        const MULTIPLIER: f64 = 4_294_967_296.0 / 600.0; // 2^32 / 600
        StoredF64::from(*difficulty * MULTIPLIER)
    }
}

pub struct OneMinusPpm;

impl UnaryTransform<PartsPerMillion32, PartsPerMillion32> for OneMinusPpm {
    #[inline(always)]
    fn apply(value: PartsPerMillion32) -> PartsPerMillion32 {
        PartsPerMillion32::ONE - value
    }
}

pub struct VBytesToWeight;

impl UnaryTransform<StoredU64, Weight64> for VBytesToWeight {
    #[inline(always)]
    fn apply(vbytes: StoredU64) -> Weight64 {
        Weight64::from(VSize::new(*vbytes))
    }
}

pub struct WeightToVSize;

impl UnaryTransform<Weight, VSize> for WeightToVSize {
    #[inline(always)]
    fn apply(weight: Weight) -> VSize {
        VSize::from(weight)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn odds_are_the_ratio_to_the_complement() {
        assert_eq!(OddsF64::apply(StoredF64::from(0.0)), StoredF64::from(0.0));
        assert_eq!(OddsF64::apply(StoredF64::from(0.5)), StoredF64::from(1.0));
        assert_eq!(OddsF64::apply(StoredF64::from(0.75)), StoredF64::from(3.0));
        assert_eq!(OddsF64::apply(StoredF64::from(1.0)), StoredF64::NAN);
    }
}
