use std::{
    fmt::{Display, Formatter, Result},
    ops::{Add, AddAssign, Sub, SubAssign},
};

use brk_types::CheckedSub;
use derive_more::Deref;
use itoa::Buffer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::unlikely;

#[cfg(feature = "storage")]
use vecdb::{Formattable, Pco};

/// Declares a fixed-point parts-per-million type over `$repr` with `$nan` as its NaN sentinel,
/// plus the mechanics all of them share: sentinel-propagating arithmetic, integer and float
/// conversions, and integer formatting (`null` in JSON, an empty CSV cell when undefined).
/// Float-to-fixed conversion (saturation) and type-specific operations stay beside each type.
macro_rules! parts_per_million {
    ($(#[$meta:meta])* $name:ident($repr:ty), nan: $nan:expr) => {
        $(#[$meta])*
        #[derive(
            Debug,
            Deref,
            Clone,
            Default,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize,
            JsonSchema,
        )]
        #[cfg_attr(feature = "storage", derive(Pco))]
        pub struct $name($repr);

        impl $name {
            pub const NAN: Self = Self($nan);

            #[inline]
            pub const fn new(value: $repr) -> Self {
                debug_assert!(value != $nan, "the NaN sentinel is reserved");
                Self(value)
            }

            #[inline]
            pub(crate) fn is_nan(self) -> bool {
                self.0 == $nan
            }

            #[inline]
            pub fn to_f32(self) -> f32 {
                if unlikely(self.is_nan()) {
                    f32::NAN
                } else {
                    self.0 as f32 / 1_000_000.0
                }
            }
        }

        impl From<usize> for $name {
            #[inline]
            fn from(value: usize) -> Self {
                debug_assert!(
                    <$repr>::try_from(value).is_ok_and(|value| value != $nan),
                    "usize out of {} range: {value}",
                    stringify!($name)
                );
                Self(value as $repr)
            }
        }

        impl From<$repr> for $name {
            #[inline]
            fn from(value: $repr) -> Self {
                Self::new(value)
            }
        }

        impl From<$name> for $repr {
            #[inline]
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl From<f32> for $name {
            #[inline]
            fn from(value: f32) -> Self {
                Self::from(value as f64)
            }
        }

        impl From<$name> for f64 {
            #[inline]
            fn from(value: $name) -> Self {
                if unlikely(value.is_nan()) {
                    f64::NAN
                } else {
                    value.0 as f64 / 1_000_000.0
                }
            }
        }

        impl From<$name> for f32 {
            #[inline]
            fn from(value: $name) -> Self {
                value.to_f32()
            }
        }

        impl Add for $name {
            type Output = Self;

            #[inline]
            fn add(self, rhs: Self) -> Self::Output {
                if unlikely(self.is_nan() || rhs.is_nan()) {
                    Self::NAN
                } else {
                    Self(self.0 + rhs.0)
                }
            }
        }

        impl AddAssign for $name {
            #[inline]
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs;
            }
        }

        impl Sub for $name {
            type Output = Self;

            #[inline]
            fn sub(self, rhs: Self) -> Self::Output {
                if unlikely(self.is_nan() || rhs.is_nan()) {
                    Self::NAN
                } else {
                    Self(self.0 - rhs.0)
                }
            }
        }

        impl CheckedSub for $name {
            fn checked_sub(self, rhs: Self) -> Option<Self> {
                if unlikely(self.is_nan() || rhs.is_nan()) {
                    Some(Self::NAN)
                } else {
                    self.0.checked_sub(rhs.0).map(Self)
                }
            }
        }


        impl Display for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result {
                f.write_str(Buffer::new().format(self.0))
            }
        }

        #[cfg(feature = "storage")]
        impl Formattable for $name {
            const UNDEFINED: bool = true;

            #[inline(always)]
            fn write_to(&self, buf: &mut Vec<u8>) {
                buf.extend_from_slice(Buffer::new().format(self.0).as_bytes());
            }

            #[inline(always)]
            fn fmt_json(&self, buf: &mut Vec<u8>) {
                if unlikely(self.is_nan()) {
                    buf.extend_from_slice(b"null");
                } else {
                    self.write_to(buf);
                }
            }

            /// Undefined: an empty CSV cell (JSON writes `null`).
            #[inline(always)]
            fn fmt_csv(&self, buf: &mut Vec<u8>) {
                if !unlikely(self.is_nan()) {
                    self.write_to(buf);
                }
            }
        }
    };
}

parts_per_million!(
    /// Unsigned parts per million stored as u32.
    /// One unit is 0.000001. Range: 0–4,294.967294.
    /// Use for precise bounded ratios and percentages.
    /// `u32::MAX` is reserved as a NaN sentinel.
    PartsPerMillion32(u32), nan: u32::MAX
);

impl PartsPerMillion32 {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(1_000_000);
}

impl From<f64> for PartsPerMillion32 {
    #[inline]
    fn from(value: f64) -> Self {
        if unlikely(!value.is_finite()) {
            return Self::NAN;
        }
        let scaled = (value * 1_000_000.0)
            .round()
            .clamp(0.0, u32::MAX as f64 - 1.0);
        Self(scaled as u32)
    }
}

parts_per_million!(
    /// Unsigned parts per million stored as u64.
    /// One unit is 0.000001. Range: 0–18,446,744,073,709.551614.
    /// Use for precise wide-range ratios.
    /// `u64::MAX` is reserved as a NaN sentinel.
    PartsPerMillion64(u64), nan: u64::MAX
);

impl PartsPerMillion64 {
    #[inline(always)]
    pub(crate) const fn inner(self) -> u64 {
        self.0
    }
}

impl From<f64> for PartsPerMillion64 {
    #[inline]
    fn from(value: f64) -> Self {
        if unlikely(!value.is_finite()) {
            return Self::NAN;
        }
        let scaled = (value * 1_000_000.0).round();
        if scaled <= 0.0 {
            Self(0)
        } else if scaled >= u64::MAX as f64 {
            Self(u64::MAX - 1)
        } else {
            Self(scaled as u64)
        }
    }
}

parts_per_million!(
    /// Signed parts per million stored as i32.
    /// One unit is 0.000001. Range: -2,147.483647 to +2,147.483647.
    /// Use for precise bounded signed ratios and percentages.
    /// `i32::MIN` is reserved as a NaN sentinel.
    PartsPerMillionSigned32(i32), nan: i32::MIN
);

impl PartsPerMillionSigned32 {
    pub const ZERO: Self = Self(0);
}

impl From<f64> for PartsPerMillionSigned32 {
    #[inline]
    fn from(value: f64) -> Self {
        if unlikely(!value.is_finite()) {
            return Self::NAN;
        }
        let scaled = (value * 1_000_000.0)
            .round()
            .clamp(i32::MIN as f64 + 1.0, i32::MAX as f64);
        Self(scaled as i32)
    }
}

impl SubAssign for PartsPerMillionSigned32 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

parts_per_million!(
    /// Signed parts per million stored as i64.
    /// One unit is 0.000001. Range: -9,223,372,036,854.775807 to +9,223,372,036,854.775807.
    /// Use for precise wide-range signed ratios and percentages.
    /// `i64::MIN` is reserved as a NaN sentinel.
    PartsPerMillionSigned64(i64), nan: i64::MIN
);

impl From<f64> for PartsPerMillionSigned64 {
    #[inline]
    fn from(value: f64) -> Self {
        if unlikely(!value.is_finite()) {
            return Self::NAN;
        }
        let scaled = (value * 1_000_000.0).round();
        if scaled <= i64::MIN as f64 {
            Self(i64::MIN + 1)
        } else if scaled >= i64::MAX as f64 {
            Self(i64::MAX)
        } else {
            Self(scaled as i64)
        }
    }
}

impl SubAssign for PartsPerMillionSigned64 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}
