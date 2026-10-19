// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::f32::consts::LN_2;

use standard_bloom::Builder as StandardBloomFilterBuilder;

pub mod bit_array;
pub mod standard_bloom;

/// Controls the size of Bloom filters written into tables.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum BloomConstructionPolicy {
    /// Uses a fixed number of Bloom-filter bits for every key.
    BitsPerKey(f32),
    /// Sizes the Bloom filter for the requested false-positive rate.
    FalsePositiveRate(f32),
}

impl Default for BloomConstructionPolicy {
    fn default() -> Self {
        Self::BitsPerKey(10.0)
    }
}

impl BloomConstructionPolicy {
    /// Creates a filter builder sized for `n` keys.
    #[must_use]
    pub(crate) fn init(self, n: usize) -> StandardBloomFilterBuilder {
        assert!(n > 0 && self.is_active());
        let (bytes, k) = self.sizing(n);
        StandardBloomFilterBuilder::new(bytes, k)
    }

    /// Returns whether this policy enables Bloom-filter construction.
    #[must_use]
    pub(crate) fn is_active(self) -> bool {
        match self {
            Self::BitsPerKey(bpk) => bpk > 0.0,
            Self::FalsePositiveRate(fpr) => fpr > 0.0,
        }
    }

    /// Returns the smallest key count whose filter fills `partition_size` bytes,
    /// computed once per writer instead of per key. Policies below one bit per key never fill
    /// a partition.
    #[must_use]
    pub(crate) fn partition_key_limit(self, partition_size: u32) -> usize {
        // Bits per key before sizing rounds up to whole bytes.
        let bits_per_key = match self {
            Self::BitsPerKey(bpk) => f64::from(bpk),
            Self::FalsePositiveRate(fpr) => -f64::from(fpr.ln()) / f64::from(LN_2).powi(2),
        };
        if !self.is_active() || bits_per_key < 1.0 {
            return usize::MAX;
        }
        let partition_size = partition_size as usize;
        // A filter never exceeds `bits_per_key * n / 8` bytes plus two bytes of rounding, so no
        // smaller count fills a partition; the threshold lies a few counts above this bound.
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_precision_loss,
            clippy::cast_sign_loss,
            reason = "a positive bound far below usize::MAX"
        )]
        let lower =
            ((8 * partition_size).saturating_sub(16) as f64 / (bits_per_key * 1.001)) as usize;
        // With at least one bit per key, `8 * partition_size` keys always fill a partition.
        (lower.max(1)..=8 * partition_size)
            .find(|&n| self.sizing(n).0 >= partition_size)
            .unwrap_or(usize::MAX)
    }

    /// The bit array's size in bytes and the hash count of a filter for `n` keys: the one sizing
    /// rule construction and partition planning share.
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        reason = "positive bit counts far below usize::MAX"
    )]
    fn sizing(self, n: usize) -> (usize, usize) {
        match self {
            Self::BitsPerKey(bpk) => {
                // NOTE: Round up so a fractional bpk never yields zero bits
                let bits = (n as f64 * f64::from(bpk)).ceil() as usize;
                (bits.div_ceil(8), hash_count(bpk))
            }
            Self::FalsePositiveRate(fpr) => {
                // NOTE: Some sensible minimum
                let fpr = fpr.max(0.000_000_1);
                // m = -n ln(fpr) / ln(2)^2 bits, rounded up to whole bytes
                let bits = -(n as f32 * fpr.ln()) / LN_2.powi(2);
                let bytes = (bits / 8.0).ceil() as usize;
                (bytes, hash_count((bytes * 8 / n) as f32))
            }
        }
    }
}

/// `bpk * ln(2)` hash functions, at least one.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "bpk easily fits into usize and LN_2 < 1.0"
)]
fn hash_count(bpk: f32) -> usize {
    ((bpk * LN_2) as usize).max(1)
}
