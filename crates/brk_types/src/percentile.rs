use crate::VSize;
use vecdb::{ColumnId, VecValue, Version};

/// Standard percentile values used throughout BRK.
pub const PERCENTILES: [u8; 19] = [
    5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55, 60, 65, 70, 75, 80, 85, 90, 95,
];

/// Length of the PERCENTILES array.
pub const PERCENTILES_LEN: usize = PERCENTILES.len();

/// Percentiles used by the rarity meter, in physical column order.
pub const RARITY_PERCENTILES: [f64; 19] = [
    0.001, 0.005, 0.01, 0.02, 0.05, 0.10, 0.20, 0.30, 0.40, 0.50, 0.60, 0.70, 0.80, 0.90, 0.95,
    0.98, 0.99, 0.995, 0.999,
];

pub const RARITY_PERCENTILES_LEN: usize = RARITY_PERCENTILES.len();

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum PercentileId {
    Pct05,
    Pct10,
    Pct15,
    Pct20,
    Pct25,
    Pct30,
    Pct35,
    Pct40,
    Pct45,
    Pct50,
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
    Pct50,
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

pub const RARITY_PERCENTILE_IDS: [RarityPercentileId; RARITY_PERCENTILES_LEN] = [
    RarityPercentileId::Pct0_1,
    RarityPercentileId::Pct0_5,
    RarityPercentileId::Pct1,
    RarityPercentileId::Pct2,
    RarityPercentileId::Pct5,
    RarityPercentileId::Pct10,
    RarityPercentileId::Pct20,
    RarityPercentileId::Pct30,
    RarityPercentileId::Pct40,
    RarityPercentileId::Pct50,
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

pub const PERCENTILE_IDS: [PercentileId; PERCENTILES_LEN] = [
    PercentileId::Pct05,
    PercentileId::Pct10,
    PercentileId::Pct15,
    PercentileId::Pct20,
    PercentileId::Pct25,
    PercentileId::Pct30,
    PercentileId::Pct35,
    PercentileId::Pct40,
    PercentileId::Pct45,
    PercentileId::Pct50,
    PercentileId::Pct55,
    PercentileId::Pct60,
    PercentileId::Pct65,
    PercentileId::Pct70,
    PercentileId::Pct75,
    PercentileId::Pct80,
    PercentileId::Pct85,
    PercentileId::Pct90,
    PercentileId::Pct95,
];

impl PercentileId {
    #[inline]
    pub const fn percentile(self) -> u8 {
        PERCENTILES[self as usize]
    }
}

impl RarityPercentileId {
    #[inline]
    pub const fn percentile(self) -> f64 {
        RARITY_PERCENTILES[self as usize]
    }
}

impl ColumnId for PercentileId {
    type Row<T>
        = [T; PERCENTILES_LEN]
    where
        T: VecValue;

    const VERSION: Version = Version::ONE;
    const ALL: &'static [Self] = &PERCENTILE_IDS;

    #[inline]
    fn index(self) -> usize {
        self as usize
    }

    #[inline]
    fn get<T: VecValue>(self, row: &Self::Row<T>) -> &T {
        &row[self.index()]
    }

    #[inline]
    fn get_mut<T: VecValue>(self, row: &mut Self::Row<T>) -> &mut T {
        &mut row[self.index()]
    }

    #[inline]
    fn from_fn<T, F>(f: F) -> Self::Row<T>
    where
        T: VecValue,
        F: FnMut(Self) -> T,
    {
        PERCENTILE_IDS.map(f)
    }

    #[inline]
    fn map<T, U, F>(row: Self::Row<T>, f: F) -> Self::Row<U>
    where
        T: VecValue,
        U: VecValue,
        F: FnMut(T) -> U,
    {
        row.map(f)
    }
}

impl ColumnId for RarityPercentileId {
    type Row<T>
        = [T; RARITY_PERCENTILES_LEN]
    where
        T: VecValue;

    const VERSION: Version = Version::ONE;
    const ALL: &'static [Self] = &RARITY_PERCENTILE_IDS;

    #[inline]
    fn index(self) -> usize {
        self as usize
    }

    #[inline]
    fn get<T: VecValue>(self, row: &Self::Row<T>) -> &T {
        &row[self.index()]
    }

    #[inline]
    fn get_mut<T: VecValue>(self, row: &mut Self::Row<T>) -> &mut T {
        &mut row[self.index()]
    }

    #[inline]
    fn from_fn<T, F>(f: F) -> Self::Row<T>
    where
        T: VecValue,
        F: FnMut(Self) -> T,
    {
        RARITY_PERCENTILE_IDS.map(f)
    }

    #[inline]
    fn map<T, U, F>(row: Self::Row<T>, f: F) -> Self::Row<U>
    where
        T: VecValue,
        U: VecValue,
        F: FnMut(T) -> U,
    {
        row.map(f)
    }
}

/// Get a percentile value from a sorted slice using nearest-rank method.
///
/// # Panics
/// Panics if the slice is empty.
pub fn get_percentile<T: Clone>(sorted: &[T], percentile: f64) -> T {
    let len = sorted.len();
    assert!(len > 0, "Cannot get percentile from empty slice");
    let index = ((len - 1) as f64 * percentile).round() as usize;
    sorted[index].clone()
}

/// Get a percentile value from a sorted (value, vsize) slice using
/// vsize-weighted interpolation — matches mempool.space's feeRange calculation.
///
/// Walks through the sorted pairs accumulating vsize. When cumulative vsize
/// crosses `total_vsize * percentile`, returns that value.
///
/// # Panics
/// Panics if the slice is empty.
pub fn get_weighted_percentile<T: Clone>(sorted_with_vsizes: &[(T, VSize)], percentile: f64) -> T {
    assert!(
        !sorted_with_vsizes.is_empty(),
        "Cannot get percentile from empty slice"
    );
    let total: u64 = sorted_with_vsizes.iter().map(|(_, v)| u64::from(*v)).sum();
    let target = (total as f64 * percentile).round() as u64;
    let mut cumulative = 0u64;
    for (value, vsize) in sorted_with_vsizes {
        cumulative += u64::from(*vsize);
        if cumulative >= target {
            return value.clone();
        }
    }
    sorted_with_vsizes.last().unwrap().0.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_ids_match_values_and_storage_order() {
        for (index, id) in PERCENTILE_IDS.into_iter().enumerate() {
            assert_eq!(id.index(), index);
            assert_eq!(id.percentile(), PERCENTILES[index]);
        }
    }

    #[test]
    fn rarity_percentile_ids_match_values_and_storage_order() {
        for (index, id) in RARITY_PERCENTILE_IDS.into_iter().enumerate() {
            assert_eq!(id.index(), index);
            assert_eq!(id.percentile(), RARITY_PERCENTILES[index]);
        }
    }
}
