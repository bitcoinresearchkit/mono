/// Standard percentile values used throughout BRK.
pub const PERCENTILES: [u8; 19] = [
    5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55, 60, 65, 70, 75, 80, 85, 90, 95,
];

/// Length of the PERCENTILES array.
pub const PERCENTILES_LEN: usize = PERCENTILES.len();

/// Percentiles used by the rarity meter, in ascending percentile order.
pub const RARITY_PERCENTILES: [f64; 19] = [
    0.001, 0.005, 0.01, 0.02, 0.05, 0.10, 0.20, 0.30, 0.40, 0.50, 0.60, 0.70, 0.80, 0.90, 0.95,
    0.98, 0.99, 0.995, 0.999,
];

pub const RARITY_PERCENTILES_LEN: usize = RARITY_PERCENTILES.len();

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum PercentileId {
    Pct5,
    Pct10,
    Pct15,
    Pct20,
    Pct25,
    Pct30,
    Pct35,
    Pct40,
    Pct45,
    Median,
    Pct55,
    Pct60,
    Pct65,
    Pct70,
    Pct75,
    Pct80,
    Pct85,
    Pct90,
    Pct95,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum RarityPercentileId {
    Pct0_1,
    Pct0_5,
    Pct1,
    Pct2,
    Pct5,
    Pct10,
    Pct20,
    Pct30,
    Pct40,
    Median,
    Pct60,
    Pct70,
    Pct80,
    Pct90,
    Pct95,
    Pct98,
    Pct99,
    Pct99_5,
    Pct99_9,
}

const RARITY_PERCENTILE_IDS: [RarityPercentileId; RARITY_PERCENTILES_LEN] = [
    RarityPercentileId::Pct0_1,
    RarityPercentileId::Pct0_5,
    RarityPercentileId::Pct1,
    RarityPercentileId::Pct2,
    RarityPercentileId::Pct5,
    RarityPercentileId::Pct10,
    RarityPercentileId::Pct20,
    RarityPercentileId::Pct30,
    RarityPercentileId::Pct40,
    RarityPercentileId::Median,
    RarityPercentileId::Pct60,
    RarityPercentileId::Pct70,
    RarityPercentileId::Pct80,
    RarityPercentileId::Pct90,
    RarityPercentileId::Pct95,
    RarityPercentileId::Pct98,
    RarityPercentileId::Pct99,
    RarityPercentileId::Pct99_5,
    RarityPercentileId::Pct99_9,
];

impl PercentileId {
    #[inline]
    pub const fn percentile(self) -> u8 {
        PERCENTILES[self as usize]
    }

    /// Catalog key: `median` for the 50th percentile, `pct5`, `pct10`, ... otherwise.
    pub const fn suffix(self) -> &'static str {
        match self {
            Self::Pct5 => "pct5",
            Self::Pct10 => "pct10",
            Self::Pct15 => "pct15",
            Self::Pct20 => "pct20",
            Self::Pct25 => "pct25",
            Self::Pct30 => "pct30",
            Self::Pct35 => "pct35",
            Self::Pct40 => "pct40",
            Self::Pct45 => "pct45",
            Self::Median => "median",
            Self::Pct55 => "pct55",
            Self::Pct60 => "pct60",
            Self::Pct65 => "pct65",
            Self::Pct70 => "pct70",
            Self::Pct75 => "pct75",
            Self::Pct80 => "pct80",
            Self::Pct85 => "pct85",
            Self::Pct90 => "pct90",
            Self::Pct95 => "pct95",
        }
    }
}

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

    #[inline]
    pub const fn percentile(self) -> f64 {
        RARITY_PERCENTILES[self as usize]
    }

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

    pub const fn suffix(self) -> &'static str {
        match self {
            Self::Pct0_1 => "pct0_1",
            Self::Pct0_5 => "pct0_5",
            Self::Pct1 => "pct1",
            Self::Pct2 => "pct2",
            Self::Pct5 => "pct5",
            Self::Pct10 => "pct10",
            Self::Pct20 => "pct20",
            Self::Pct30 => "pct30",
            Self::Pct40 => "pct40",
            Self::Median => "median",
            Self::Pct60 => "pct60",
            Self::Pct70 => "pct70",
            Self::Pct80 => "pct80",
            Self::Pct90 => "pct90",
            Self::Pct95 => "pct95",
            Self::Pct98 => "pct98",
            Self::Pct99 => "pct99",
            Self::Pct99_5 => "pct99_5",
            Self::Pct99_9 => "pct99_9",
        }
    }
}

impl RarityPercentileId {
    #[inline]
    pub fn from_fn<T, F>(f: F) -> [T; RARITY_PERCENTILES_LEN]
    where
        F: FnMut(Self) -> T,
    {
        RARITY_PERCENTILE_IDS.map(f)
    }
}
