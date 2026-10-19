/// Generates a scalar newtype over a primitive with storage, serde, schema and formatting.
///
/// `int` for unsigned and signed integers, `float` for `f32`/`f64`. Narrowing conversions are
/// checked in dev builds; in release, integers saturate and an oversized `f64` becomes an infinite
/// `f32`. Floats order NaN first and equal to itself, and write non-finite values as JSON `null` /
/// an empty CSV cell (an undefined value: `Formattable::UNDEFINED`).
macro_rules! scalar {
    ($(#[$meta:meta])* int $name:ident($repr:ty)) => {
        $crate::scalar::scalar!(@int $(#[$meta])* $name($repr));

        impl ::std::iter::Sum for $name {
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                Self(iter.map(|value| value.0).sum())
            }
        }

        /// Saturating (`as` from a float clamps and maps NaN to zero; out of range is checked in
        /// dev builds).
        impl From<f64> for $name {
            #[inline]
            fn from(value: f64) -> Self {
                debug_assert!(
                    value.is_nan() || (<$repr>::MIN as f64..=<$repr>::MAX as f64).contains(&value),
                    "{value} doesn't fit {}",
                    stringify!($name)
                );
                Self(value as $repr)
            }
        }

        impl From<$name> for f64 {
            #[inline]
            fn from(value: $name) -> Self {
                value.0 as f64
            }
        }

        ::brk_types::impl_int_format!($name);
    };

    // An integer whose minimum is its undefined value, served as null.
    ($(#[$meta:meta])* nullable int $name:ident($repr:ty)) => {
        $crate::scalar::scalar!(@int $(#[$meta])* $name($repr));

        impl $name {
            /// The undefined value, served as null.
            pub const NAN: Self = Self(<$repr>::MIN);

            #[inline]
            pub fn is_nan(self) -> bool {
                self == Self::NAN
            }
        }

        /// NaN is the undefined value; a finite value must fit (checked in dev builds).
        impl From<f64> for $name {
            #[inline]
            fn from(value: f64) -> Self {
                if value.is_nan() {
                    return Self::NAN;
                }
                debug_assert!(
                    (<$repr>::MIN as f64 + 1.0..=<$repr>::MAX as f64).contains(&value),
                    "{value} doesn't fit {}",
                    stringify!($name)
                );
                Self(value as $repr)
            }
        }

        impl From<$name> for f64 {
            #[inline]
            fn from(value: $name) -> Self {
                if value.is_nan() { f64::NAN } else { value.0 as f64 }
            }
        }

        ::brk_types::impl_int_format!($name, nullable);
    };

    (@int $(#[$meta:meta])* $name:ident($repr:ty)) => {
        $(#[$meta])*
        #[derive(
            Debug,
            Default,
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            ::derive_more::Deref,
            ::serde::Serialize,
            ::serde::Deserialize,
            ::schemars::JsonSchema,
        )]
        #[cfg_attr(feature = "storage", derive(::vecdb::Pco))]
        pub struct $name($repr);

        impl $name {
            pub const ZERO: Self = Self(0);
            pub const MAX: Self = Self(<$repr>::MAX);

            #[inline]
            pub const fn new(value: $repr) -> Self {
                Self(value)
            }
        }

        impl From<$repr> for $name {
            #[inline]
            fn from(value: $repr) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $repr {
            #[inline]
            fn from(value: $name) -> Self {
                value.0
            }
        }

        /// Saturating: out-of-range values clamp, never wrap (checked in dev builds).
        impl From<usize> for $name {
            #[inline]
            fn from(value: usize) -> Self {
                debug_assert!(<$repr>::try_from(value).is_ok(), "{value} doesn't fit {}", stringify!($name));
                Self(<$repr>::try_from(value).unwrap_or(<$repr>::MAX))
            }
        }

        impl ::brk_types::CheckedSub for $name {
            #[inline]
            fn checked_sub(self, rhs: Self) -> Option<Self> {
                self.0.checked_sub(rhs.0).map(Self)
            }
        }

        impl ::std::ops::Add for $name {
            type Output = Self;
            #[inline]
            fn add(self, rhs: Self) -> Self {
                Self(self.0 + rhs.0)
            }
        }

        impl ::std::ops::AddAssign for $name {
            #[inline]
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl ::std::ops::Sub for $name {
            type Output = Self;
            #[inline]
            fn sub(self, rhs: Self) -> Self {
                Self(self.0 - rhs.0)
            }
        }

        impl ::std::ops::SubAssign for $name {
            #[inline]
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }
    };

    ($(#[$meta:meta])* float $name:ident(f32)) => {
        $crate::scalar::scalar!(@float $(#[$meta])* $name(f32));

        /// Narrowing; a finite value must fit `f32` (checked in dev builds), a non-finite one is
        /// undefined.
        impl From<f64> for $name {
            #[inline]
            fn from(value: f64) -> Self {
                debug_assert!(
                    !value.is_finite() || (f32::MIN as f64..=f32::MAX as f64).contains(&value),
                    "{value} doesn't fit {}",
                    stringify!($name)
                );
                Self(value as f32)
            }
        }

        impl From<$name> for f64 {
            #[inline]
            fn from(value: $name) -> Self {
                f64::from(value.0)
            }
        }
    };

    ($(#[$meta:meta])* float $name:ident(f64)) => {
        $crate::scalar::scalar!(@float $(#[$meta])* $name(f64));
    };

    (@float $(#[$meta:meta])* $name:ident($repr:ty)) => {
        $(#[$meta])*
        #[derive(
            Debug,
            Default,
            Clone,
            Copy,
            ::derive_more::Deref,
            ::serde::Serialize,
            ::serde::Deserialize,
            ::schemars::JsonSchema,
        )]
        #[cfg_attr(feature = "storage", derive(::vecdb::Pco))]
        pub struct $name($repr);

        impl $name {
            pub const ZERO: Self = Self(0.0);
            pub const NAN: Self = Self(<$repr>::NAN);

            #[inline]
            pub const fn new(value: $repr) -> Self {
                Self(value)
            }

            #[inline]
            pub fn is_nan(self) -> bool {
                self.0.is_nan()
            }
        }

        impl From<$repr> for $name {
            #[inline]
            fn from(value: $repr) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $repr {
            #[inline]
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl From<usize> for $name {
            #[inline]
            fn from(value: usize) -> Self {
                Self(value as $repr)
            }
        }

        impl ::brk_types::CheckedSub for $name {
            #[inline]
            fn checked_sub(self, rhs: Self) -> Option<Self> {
                Some(Self(self.0 - rhs.0))
            }
        }

        impl ::std::ops::Add for $name {
            type Output = Self;
            #[inline]
            fn add(self, rhs: Self) -> Self {
                Self(self.0 + rhs.0)
            }
        }

        impl ::std::ops::AddAssign for $name {
            #[inline]
            fn add_assign(&mut self, rhs: Self) {
                self.0 += rhs.0;
            }
        }

        impl ::std::ops::Sub for $name {
            type Output = Self;
            #[inline]
            fn sub(self, rhs: Self) -> Self {
                Self(self.0 - rhs.0)
            }
        }

        impl ::std::ops::SubAssign for $name {
            #[inline]
            fn sub_assign(&mut self, rhs: Self) {
                self.0 -= rhs.0;
            }
        }

        impl ::std::ops::Neg for $name {
            type Output = Self;
            #[inline]
            fn neg(self) -> Self {
                Self(-self.0)
            }
        }

        impl ::std::iter::Sum for $name {
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                Self(iter.map(|value| value.0).sum())
            }
        }

        ::brk_types::impl_float_ord_format!($name);
    };
}

pub(crate) use scalar;
