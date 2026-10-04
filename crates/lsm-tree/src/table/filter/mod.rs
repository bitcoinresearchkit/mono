// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

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

    /// Returns the estimated filter size in bytes.
    #[must_use]
    pub(crate) fn estimated_filter_size(self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }

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
