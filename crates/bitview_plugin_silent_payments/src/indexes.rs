use std::ops::Add;

use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{Formattable, Pco, PrintableIndex, VecIndex};

/// A row index of the plugin's own: `u32`, named for the API.
macro_rules! row_index {
    ($(#[$meta:meta])* $name:ident, $index_name:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Pco, Serialize, JsonSchema)]
        pub struct $name(u32);

        impl From<usize> for $name {
            #[inline]
            fn from(value: usize) -> Self {
                debug_assert!(u32::try_from(value).is_ok(), "{value} doesn't fit {}", stringify!($name));
                Self(value as u32)
            }
        }

        impl From<$name> for usize {
            #[inline]
            fn from(value: $name) -> Self {
                value.0 as usize
            }
        }

        impl Add<usize> for $name {
            type Output = Self;
            #[inline]
            fn add(self, rhs: usize) -> Self {
                Self::from(usize::from(self) + rhs)
            }
        }

        impl Formattable for $name {
            #[inline(always)]
            fn write_to(&self, buf: &mut Vec<u8>) {
                self.0.write_to(buf)
            }
        }

        impl PrintableIndex for $name {
            fn to_string() -> &'static str {
                $index_name
            }
        }

        impl VecIndex for $name {}
    };
}

row_index!(
    /// Index of an eligible transaction among all of them, in chain order.
    TweakIndex, "tweak_index"
);
row_index!(
    /// Index of a Taproot output of an eligible transaction among all of them, in chain order.
    TaprootPrefixIndex, "taproot_prefix_index"
);
