/// Persisted tag for XXH3-based table filters.
pub const XXH3_TAG: u8 = 0;

/// Generates a 64-bit hash using xxh3.
pub fn hash64(bytes: &[u8]) -> u64 {
    xxhash_rust::xxh3::xxh3_64(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test]
    fn test_hash64() {
        assert_eq!(16_959_823_422_411_450_475, hash64(&[0, 0, 0]));
        assert_eq!(8_004_557_073_989_523_290, hash64(&[0, 0, 1]));
    }
}
