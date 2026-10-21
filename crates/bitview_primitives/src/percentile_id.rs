//! Percentile families, each declared once as a table: `percentile_id!` generates the id enum
//! and values here, and `bitview_vecs` builds its per-percentile collections from the same rows.

/// The standard percentiles: calls `$callback! { [$args] Id, VALUES: [type; LEN]; rows }`, one
/// `Variant => suffix, value, "doc";` row per percentile in ascending order.
#[macro_export]
macro_rules! percentile_table {
    ($callback:ident $(, $arg:tt)*) => {
        $callback! {
            [$($arg)*] PercentileId, PERCENTILES: [u8; PERCENTILES_LEN];
            P5 => p5, 5, "Uses the 5th percentile.";
            P10 => p10, 10, "Uses the 10th percentile.";
            P15 => p15, 15, "Uses the 15th percentile.";
            P20 => p20, 20, "Uses the 20th percentile.";
            P25 => p25, 25, "Uses the 25th percentile.";
            P30 => p30, 30, "Uses the 30th percentile.";
            P35 => p35, 35, "Uses the 35th percentile.";
            P40 => p40, 40, "Uses the 40th percentile.";
            P45 => p45, 45, "Uses the 45th percentile.";
            Median => median, 50, "Uses the 50th percentile (median).";
            P55 => p55, 55, "Uses the 55th percentile.";
            P60 => p60, 60, "Uses the 60th percentile.";
            P65 => p65, 65, "Uses the 65th percentile.";
            P70 => p70, 70, "Uses the 70th percentile.";
            P75 => p75, 75, "Uses the 75th percentile.";
            P80 => p80, 80, "Uses the 80th percentile.";
            P85 => p85, 85, "Uses the 85th percentile.";
            P90 => p90, 90, "Uses the 90th percentile.";
            P95 => p95, 95, "Uses the 95th percentile.";
        }
    };
}

/// The rarity meter's percentiles, in ascending order; rows as in [`percentile_table!`].
#[macro_export]
macro_rules! rarity_percentile_table {
    ($callback:ident $(, $arg:tt)*) => {
        $callback! {
            [$($arg)*] RarityPercentileId, RARITY_PERCENTILES: [f64; RARITY_PERCENTILES_LEN];
            P0_1 => p0_1, 0.001, "Uses the 0.1% quantile.";
            P0_5 => p0_5, 0.005, "Uses the 0.5% quantile.";
            P1 => p1, 0.01, "Uses the 1% quantile.";
            P2 => p2, 0.02, "Uses the 2% quantile.";
            P5 => p5, 0.05, "Uses the 5% quantile.";
            P10 => p10, 0.10, "Uses the 10% quantile.";
            P20 => p20, 0.20, "Uses the 20% quantile.";
            P30 => p30, 0.30, "Uses the 30% quantile.";
            P40 => p40, 0.40, "Uses the 40% quantile.";
            Median => median, 0.50, "Uses the 50% quantile.";
            P60 => p60, 0.60, "Uses the 60% quantile.";
            P70 => p70, 0.70, "Uses the 70% quantile.";
            P80 => p80, 0.80, "Uses the 80% quantile.";
            P90 => p90, 0.90, "Uses the 90% quantile.";
            P95 => p95, 0.95, "Uses the 95% quantile.";
            P98 => p98, 0.98, "Uses the 98% quantile.";
            P99 => p99, 0.99, "Uses the 99% quantile.";
            P99_5 => p99_5, 0.995, "Uses the 99.5% quantile.";
            P99_9 => p99_9, 0.999, "Uses the 99.9% quantile.";
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

            /// Catalog key: `median` for the 50th percentile, `p5`, `p10`, ... otherwise.
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
        Self::P0_1,
        Self::P0_5,
        Self::P1,
        Self::P2,
        Self::P5,
        Self::P95,
        Self::P98,
        Self::P99,
        Self::P99_5,
        Self::P99_9,
    ];

    pub const fn boundary_index(self) -> Option<usize> {
        match self {
            Self::P0_1 => Some(0),
            Self::P0_5 => Some(1),
            Self::P1 => Some(2),
            Self::P2 => Some(3),
            Self::P5 => Some(4),
            Self::P95 => Some(5),
            Self::P98 => Some(6),
            Self::P99 => Some(7),
            Self::P99_5 => Some(8),
            Self::P99_9 => Some(9),
            _ => None,
        }
    }

    pub const fn is_lower_boundary(self) -> bool {
        matches!(
            self,
            Self::P0_1 | Self::P0_5 | Self::P1 | Self::P2 | Self::P5
        )
    }
}
