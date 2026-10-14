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
        match self {
            Self::BitsPerKey(bpk) => StandardBloomFilterBuilder::with_bpk(n, bpk),
            Self::FalsePositiveRate(fpr) => StandardBloomFilterBuilder::with_fp_rate(n, fpr),
        }
    }

    /// Returns whether this policy enables Bloom-filter construction.
    #[must_use]
    pub(crate) fn is_active(self) -> bool {
        match self {
            Self::BitsPerKey(bpk) => bpk > 0.0,
            Self::FalsePositiveRate(fpr) => fpr > 0.0,
        }
    }

    /// Returns the smallest key count whose estimated filter fills `partition_size` bytes,
    /// computed once per writer instead of per key. Policies below one bit per key never fill
    /// a partition.
    #[must_use]
    pub(crate) fn partition_key_limit(self, partition_size: u32) -> usize {
        // Bits per key before the estimate rounds to whole bytes.
        let bits_per_key = match self {
            Self::BitsPerKey(bpk) => f64::from(bpk),
            Self::FalsePositiveRate(fpr) => -f64::from(fpr.ln()) / f64::from(LN_2).powi(2),
        };
        if !self.is_active() || bits_per_key < 1.0 {
            return usize::MAX;
        }
        let partition_size = partition_size as usize;
        // The estimate never exceeds `bits_per_key * n / 8` bytes plus one byte of rounding, so
        // no smaller count fills a partition; the threshold lies a few counts above this bound.
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
            .find(|&n| self.estimated_filter_size(n) >= partition_size)
            .unwrap_or(usize::MAX)
    }

    /// Returns the estimated filter size in bytes.
    #[must_use]
    fn estimated_filter_size(self, n: usize) -> usize {
        #[expect(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "this positive estimate intentionally floors to a whole number of bytes"
        )]
        match self {
            Self::BitsPerKey(bpk) => (bpk * (n as f32)) as usize / 8,
            Self::FalsePositiveRate(fpr) => {
                let m = StandardBloomFilterBuilder::calculate_m(n, fpr);
                let bpk = (m / n) as f32;
                (bpk * (n as f32)) as usize / 8
            }
        }
    }
}
