// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::f32::consts::LN_2;

use xxhash_rust::xxh3::xxh3_64;

use super::super::bit_array::Builder as BitArrayBuilder;

pub fn secondary_hash(h1: u64) -> u64 {
    // Taken from https://github.com/tomtomwombat/fastbloom
    h1.wrapping_shr(32).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95)
}

#[derive(Debug)]
pub struct Builder {
    /// Raw bytes exposed as bit array
    inner: BitArrayBuilder,

    /// Bit count
    pub m: usize,

    /// Number of hash functions
    pub k: usize,
}

impl Builder {
    #[must_use]
    pub fn build(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(2 * size_of::<u64>() + self.inner.bytes().len());
        bytes.extend_from_slice(&(self.m as u64).to_le_bytes());
        bytes.extend_from_slice(&(self.k as u64).to_le_bytes());
        bytes.extend_from_slice(self.inner.bytes());
        bytes
    }

    /// Constructs a bloom filter that can hold `n` items
    /// while maintaining a certain false positive rate `fpr`.
    #[must_use]
    pub fn with_fp_rate(n: usize, fpr: f32) -> Self {
        assert!(n > 0);

        // NOTE: Some sensible minimum
        let fpr = fpr.max(0.000_000_1);

        let m = Self::calculate_m(n, fpr);

        #[expect(
            clippy::cast_precision_loss,
            reason = "bpk tends to be in the range of 0-50, so easily fits into u32"
        )]
        let bpk = (m / n) as f32;

        #[expect(
            clippy::cast_sign_loss,
            clippy::cast_possible_truncation,
            reason = "bpk easily fits into u32 and LN_2 < 1.0, so should still fit into a usize as well"
        )]
        let k = ((bpk * LN_2) as usize).max(1);

        Self {
            inner: BitArrayBuilder::with_capacity(m / 8),
            m,
            k,
        }
    }

    /// Constructs a bloom filter that can hold `n` items
    /// with `bpk` bits per key.
    ///
    /// 10 bits per key is a sensible default.
    #[must_use]
    pub fn with_bpk(n: usize, bpk: f32) -> Self {
        assert!(bpk > 0.0);
        assert!(n > 0);

        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "bpk tends to be in the range of 0-50, so easily fits into usize"
        )]
        let m = n * (bpk as usize);

        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "bpk easily fits into usize and LN_2 < 1.0, so should still fit into a usize as well"
        )]
        let k = ((bpk * LN_2) as usize).max(1);

        // NOTE: Round up so we don't get too little bits
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss,
            reason = "m already fits, and because we divide, it should definitely fit into usize"
        )]
        let bytes = (m as f32 / 8.0).ceil() as usize;

        Self {
            inner: BitArrayBuilder::with_capacity(bytes),
            m: bytes * 8,
            k,
        }
    }

    pub fn calculate_m(n: usize, fp_rate: f32) -> usize {
        #[expect(
            clippy::cast_precision_loss,
            reason = "n tends to be in the single millions at most, so f32 should be precise enough"
        )]
        let n = n as f32;
        let ln2_squared = LN_2.powi(2);

        let numerator = n * fp_rate.ln();
        let m = -(numerator / ln2_squared);

        // NOTE: Round up to next byte
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "m already fits, and because we divide, it should definitely fit into usize"
        )]
        let result = ((m / 8.0).ceil() * 8.0) as usize;
        result
    }

    /// Adds the key to the filter.
    pub fn set_with_hash(&mut self, mut h1: u64) {
        let mut h2 = secondary_hash(h1);

        for i in 1..=(self.k as u64) {
            let idx = h1 % (self.m as u64);

            #[expect(
                clippy::cast_possible_truncation,
                reason = "filters tend to be pretty small, definitely less than 4 GiB, even for large tables"
            )]
            self.inner.enable_bit(idx as usize);

            h1 = h1.wrapping_add(h2);
            h2 = h2.wrapping_mul(i);
        }
    }

    /// Gets the hash of a key.
    #[must_use]
    pub fn get_hash(key: &[u8]) -> u64 {
        xxh3_64(key)
    }
}
