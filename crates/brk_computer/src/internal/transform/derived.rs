use std::marker::PhantomData;

use brk_types::{Cents, PartsPerMillionSigned64, StoredF32};
use vecdb::{BinaryTransform, UnaryTransform};

use crate::internal::FixedRatio;

pub struct DaysToYears;

impl UnaryTransform<StoredF32, StoredF32> for DaysToYears {
    #[inline(always)]
    fn apply(v: StoredF32) -> StoredF32 {
        StoredF32::from(*v / 365.0)
    }
}

pub struct Cagr<const YEARS: u8>;

impl<const YEARS: u8> UnaryTransform<PartsPerMillionSigned64, PartsPerMillionSigned64>
    for Cagr<YEARS>
{
    #[inline(always)]
    fn apply(value: PartsPerMillionSigned64) -> PartsPerMillionSigned64 {
        let ratio = f64::from(value);
        PartsPerMillionSigned64::from((ratio + 1.0).powf(1.0 / YEARS as f64) - 1.0)
    }
}

pub trait SqrtDays {
    const FACTOR: f32;
}

pub struct Days1;
impl SqrtDays for Days1 {
    const FACTOR: f32 = 1.0; // 1.0_f32.sqrt()
}

pub struct Days7;
impl SqrtDays for Days7 {
    const FACTOR: f32 = 2.6457513; // 7.0_f32.sqrt()
}

pub struct Days30;
impl SqrtDays for Days30 {
    const FACTOR: f32 = 5.477226; // 30.0_f32.sqrt()
}

pub struct Days365;
impl SqrtDays for Days365 {
    const FACTOR: f32 = 19.104973; // 365.0_f32.sqrt()
}

pub struct TimesSqrt<D: SqrtDays>(PhantomData<D>);

impl<D: SqrtDays> UnaryTransform<StoredF32, StoredF32> for TimesSqrt<D> {
    #[inline(always)]
    fn apply(v: StoredF32) -> StoredF32 {
        (*v * D::FACTOR).into()
    }
}

pub struct PriceTimesRatio<R>(PhantomData<R>);

impl<R: FixedRatio> BinaryTransform<Cents, R, Cents> for PriceTimesRatio<R> {
    #[inline(always)]
    fn apply(price: Cents, ratio: R) -> Cents {
        Cents::from(f64::from(price) * ratio.into())
    }
}
