// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

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

    /// An empty filter of `bytes` bytes (`8 * bytes` bits) and `k` hash functions.
    #[must_use]
    pub fn new(bytes: usize, k: usize) -> Self {
        Self {
            inner: BitArrayBuilder::with_capacity(bytes),
            m: bytes * 8,
            k,
        }
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
