/// `Display` and `Formattable` for an integer newtype `$t(int)`: the integer's decimal digits.
///
/// The `vecdb` impl follows the calling crate's `storage` feature, which must enable `vecdb`; the
/// calling crate gets `itoa` through `brk_types`.
#[macro_export]
macro_rules! impl_int_format {
    ($t:ty) => {
        impl ::std::fmt::Display for $t {
            #[inline]
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str($crate::__private::itoa::Buffer::new().format(self.0))
            }
        }

        #[cfg(feature = "storage")]
        impl vecdb::Formattable for $t {
            #[inline(always)]
            fn write_to(&self, buf: &mut Vec<u8>) {
                vecdb::Formattable::write_to(&self.0, buf)
            }
        }
    };
}

/// The impls every integer index newtype (`struct $t($inner)`) shares: arithmetic with itself and
/// `usize`, checked subtraction, lossless conversions with `$inner` and `usize`, and integer
/// formatting. Sentinels, narrowing conversions, capacity and aliases stay on each type.
///
/// The `vecdb` impls follow the calling crate's `storage` feature, which must enable `vecdb`; the
/// calling crate gets `itoa` through `brk_types`.
#[macro_export]
macro_rules! impl_integer_index {
    ($t:ident, $inner:ty) => {
        impl ::std::ops::Add<$t> for $t {
            type Output = Self;
            #[inline]
            fn add(self, rhs: $t) -> Self::Output {
                Self(self.0 + rhs.0)
            }
        }

        impl ::std::ops::Add<usize> for $t {
            type Output = Self;
            #[inline]
            fn add(self, rhs: usize) -> Self::Output {
                Self(self.0 + rhs as $inner)
            }
        }

        impl ::std::ops::AddAssign<$t> for $t {
            #[inline]
            fn add_assign(&mut self, rhs: $t) {
                self.0 += rhs.0
            }
        }

        impl $crate::CheckedSub<$t> for $t {
            #[inline]
            fn checked_sub(self, rhs: Self) -> Option<Self> {
                self.0.checked_sub(rhs.0).map(Self)
            }
        }

        impl From<$inner> for $t {
            #[inline]
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }
        impl From<$t> for $inner {
            #[inline]
            fn from(value: $t) -> Self {
                value.0
            }
        }

        impl From<usize> for $t {
            #[inline]
            fn from(value: usize) -> Self {
                Self(value as $inner)
            }
        }
        impl From<$t> for usize {
            #[inline]
            fn from(value: $t) -> Self {
                value.0 as usize
            }
        }

        $crate::impl_int_format!($t);
    };
}

/// NaN-aware total order and formatting for a float newtype `$t(float)`: NaN equals itself and
/// orders first, and a non-finite value is written as JSON `null` / an empty CSV cell (an
/// undefined value: `Formattable::UNDEFINED`).
///
/// The `vecdb` impl follows the calling crate's `storage` feature, which must enable `vecdb`; the
/// calling crate gets `ryu` through `brk_types`.
#[macro_export]
macro_rules! impl_float_ord_format {
    ($t:ty) => {
        impl PartialEq for $t {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                match (self.0.is_nan(), other.0.is_nan()) {
                    (true, true) => true,
                    (false, false) => self.0 == other.0,
                    _ => false,
                }
            }
        }

        impl Eq for $t {}

        impl PartialOrd for $t {
            #[inline]
            fn partial_cmp(&self, other: &Self) -> Option<::std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $t {
            #[inline]
            fn cmp(&self, other: &Self) -> ::std::cmp::Ordering {
                match (self.0.is_nan(), other.0.is_nan()) {
                    (true, true) => ::std::cmp::Ordering::Equal,
                    (true, false) => ::std::cmp::Ordering::Less,
                    (false, true) => ::std::cmp::Ordering::Greater,
                    (false, false) => self.0.partial_cmp(&other.0).unwrap(),
                }
            }
        }

        impl ::std::fmt::Display for $t {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str($crate::__private::ryu::Buffer::new().format(self.0))
            }
        }

        #[cfg(feature = "storage")]
        impl vecdb::Formattable for $t {
            const UNDEFINED: bool = true;

            /// Non-finite: an empty CSV cell.
            #[inline(always)]
            fn write_to(&self, buf: &mut Vec<u8>) {
                if self.0.is_finite() {
                    buf.extend_from_slice(
                        $crate::__private::ryu::Buffer::new()
                            .format_finite(self.0)
                            .as_bytes(),
                    );
                }
            }

            /// One finiteness check on the hot path.
            #[inline(always)]
            fn fmt_json(&self, buf: &mut Vec<u8>) {
                if self.0.is_finite() {
                    buf.extend_from_slice(
                        $crate::__private::ryu::Buffer::new()
                            .format_finite(self.0)
                            .as_bytes(),
                    );
                } else {
                    buf.extend_from_slice(b"null");
                }
            }
        }
    };
}
