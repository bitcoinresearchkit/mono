//! Percentile families, each declared once as a table: `percentile_id!` generates the id enum
//! and values here, and `bitview_vecs` builds its per-percentile collections from the same rows.

/// The standard percentiles: calls `$callback! { [$args] Id, VALUES: [type; LEN]; rows }`, one
/// `Variant => suffix, value, "doc";` row per percentile in ascending order.
#[macro_export]
macro_rules! percentile_table {
    ($callback:ident $(, $arg:tt)*) => {
        $callback! {
            [$($arg)*] PercentileId, PERCENTILES: [u8; PERCENTILES_LEN];
            Pct5 => pct5, 5, "Uses the 5th percentile.";
            Pct10 => pct10, 10, "Uses the 10th percentile.";
            Pct15 => pct15, 15, "Uses the 15th percentile.";
            Pct20 => pct20, 20, "Uses the 20th percentile.";
            Pct25 => pct25, 25, "Uses the 25th percentile.";
            Pct30 => pct30, 30, "Uses the 30th percentile.";
            Pct35 => pct35, 35, "Uses the 35th percentile.";
            Pct40 => pct40, 40, "Uses the 40th percentile.";
            Pct45 => pct45, 45, "Uses the 45th percentile.";
            Median => median, 50, "Uses the 50th percentile (median).";
            Pct55 => pct55, 55, "Uses the 55th percentile.";
            Pct60 => pct60, 60, "Uses the 60th percentile.";
            Pct65 => pct65, 65, "Uses the 65th percentile.";
            Pct70 => pct70, 70, "Uses the 70th percentile.";
            Pct75 => pct75, 75, "Uses the 75th percentile.";
            Pct80 => pct80, 80, "Uses the 80th percentile.";
            Pct85 => pct85, 85, "Uses the 85th percentile.";
            Pct90 => pct90, 90, "Uses the 90th percentile.";
            Pct95 => pct95, 95, "Uses the 95th percentile.";
        }
    };
}

/// The rarity meter's percentiles, in ascending order; rows as in [`percentile_table!`].
#[macro_export]
macro_rules! rarity_percentile_table {
    ($callback:ident $(, $arg:tt)*) => {
        $callback! {
            [$($arg)*] RarityPercentileId, RARITY_PERCENTILES: [f64; RARITY_PERCENTILES_LEN];
            Pct0_1 => pct0_1, 0.001, "Uses the 0.1% quantile.";
            Pct0_5 => pct0_5, 0.005, "Uses the 0.5% quantile.";
            Pct1 => pct1, 0.01, "Uses the 1% quantile.";
            Pct2 => pct2, 0.02, "Uses the 2% quantile.";
            Pct5 => pct5, 0.05, "Uses the 5% quantile.";
            Pct10 => pct10, 0.10, "Uses the 10% quantile.";
            Pct20 => pct20, 0.20, "Uses the 20% quantile.";
            Pct30 => pct30, 0.30, "Uses the 30% quantile.";
            Pct40 => pct40, 0.40, "Uses the 40% quantile.";
            Median => median, 0.50, "Uses the 50% quantile.";
            Pct60 => pct60, 0.60, "Uses the 60% quantile.";
            Pct70 => pct70, 0.70, "Uses the 70% quantile.";
            Pct80 => pct80, 0.80, "Uses the 80% quantile.";
            Pct90 => pct90, 0.90, "Uses the 90% quantile.";
            Pct95 => pct95, 0.95, "Uses the 95% quantile.";
            Pct98 => pct98, 0.98, "Uses the 98% quantile.";
            Pct99 => pct99, 0.99, "Uses the 99% quantile.";
            Pct99_5 => pct99_5, 0.995, "Uses the 99.5% quantile.";
            Pct99_9 => pct99_9, 0.999, "Uses the 99.9% quantile.";
        }
    };
}

macro_rules! percentile_id {
    (
        [] $id:ident, $values:ident: [$ty:ty; $len:ident];
        $($variant:ident => $suffix:ident, $value:expr, $doc:tt;)*
    ) => {
        /// The family's values, in ascending order.
        pub const $values: [$ty; $len] = [$($value),*];

        pub const $len: usize = [$(stringify!($variant)),*].len();

        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(u8)]
        pub enum $id {
            $($variant),*
        }

        impl $id {
            /// Every id, in ascending order.
            pub const ALL: [Self; $len] = [$(Self::$variant),*];

            #[inline]
            pub const fn percentile(self) -> $ty {
                $values[self as usize]
            }

            /// Catalog key: `median` for the 50th percentile, `pct5`, `pct10`, ... otherwise.
            pub const fn suffix(self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($suffix)),*
                }
            }

            #[inline]
            pub fn from_fn<T>(f: impl FnMut(Self) -> T) -> [T; $len] {
                Self::ALL.map(f)
            }
        }
    };
}

percentile_table!(percentile_id);
rarity_percentile_table!(percentile_id);

impl RarityPercentileId {
    pub const BOUNDARIES: [Self; 10] = [
        Self::Pct0_1,
        Self::Pct0_5,
        Self::Pct1,
        Self::Pct2,
        Self::Pct5,
        Self::Pct95,
        Self::Pct98,
        Self::Pct99,
        Self::Pct99_5,
        Self::Pct99_9,
    ];

    pub const fn boundary_index(self) -> Option<usize> {
        match self {
            Self::Pct0_1 => Some(0),
            Self::Pct0_5 => Some(1),
            Self::Pct1 => Some(2),
            Self::Pct2 => Some(3),
            Self::Pct5 => Some(4),
            Self::Pct95 => Some(5),
            Self::Pct98 => Some(6),
            Self::Pct99 => Some(7),
            Self::Pct99_5 => Some(8),
            Self::Pct99_9 => Some(9),
            _ => None,
        }
    }

    pub const fn is_lower_boundary(self) -> bool {
        matches!(
            self,
            Self::Pct0_1 | Self::Pct0_5 | Self::Pct1 | Self::Pct2 | Self::Pct5
        )
    }
}
