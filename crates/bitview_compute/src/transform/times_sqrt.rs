use brk_types::StoredF32;
use vecdb::UnaryTransform;

pub struct TimesSqrt<const DAYS: u16>;

impl<const DAYS: u16> TimesSqrt<DAYS> {
    const FACTOR: f32 = match DAYS {
        1 => 1.0,
        7 => 2.6457513,
        30 => 5.477226,
        365 => 19.104973,
        _ => panic!("unsupported square-root day count"),
    };
}

impl<const DAYS: u16> UnaryTransform<StoredF32, StoredF32> for TimesSqrt<DAYS> {
    #[inline(always)]
    fn apply(value: StoredF32) -> StoredF32 {
        (*value * Self::FACTOR).into()
    }
}
