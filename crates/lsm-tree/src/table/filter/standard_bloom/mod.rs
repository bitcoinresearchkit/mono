// Copyright (c) 2024-present, fjall-rs
// This source code is licensed under both the Apache 2.0 and MIT License
// (found in the LICENSE-* files in the repository)

use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{Error, Result, Slice};

use self::builder::secondary_hash;

pub mod builder;

pub use builder::Builder;

const HEADER_LEN: usize = 2 * size_of::<u64>();

/// Reads an owned, pinned filter or borrows a filter from a cached block.
///
/// The filter uses double hashing instead of `k` hash functions, see:
/// <https://fjall-rs.github.io/post/bloom-filter-hash-sharing>
#[derive(Clone)]
pub struct StandardBloomFilterReader<B = Slice> {
    /// Encoded header and bit array.
    inner: B,

    /// Bit count
    m: usize,

    /// Number of hash functions
    k: usize,
}

impl<B: AsRef<[u8]>> StandardBloomFilterReader<B> {
    pub fn new(slice: B) -> Result<Self> {
        let mut reader = Cursor::new(slice.as_ref());

        let m = usize::try_from(reader.read_u64::<LittleEndian>()?)
            .map_err(|_| Error::InvalidHeader("BloomFilter"))?;
        let k = usize::try_from(reader.read_u64::<LittleEndian>()?)
            .map_err(|_| Error::InvalidHeader("BloomFilter"))?;

        let bytes = slice
            .as_ref()
            .get(HEADER_LEN..)
            .ok_or(Error::InvalidHeader("BloomFilter"))?;
        let bit_len = bytes
            .len()
            .checked_mul(8)
            .ok_or(Error::InvalidHeader("BloomFilter"))?;
        if m == 0 || m != bit_len {
            return Err(Error::InvalidHeader("BloomFilter"));
        }

        Ok(Self { k, m, inner: slice })
    }

    pub fn maybe_contains(&self, key: &[u8], hash: &mut Option<u64>) -> bool {
        let hash = *hash.get_or_insert_with(|| Builder::get_hash(key));
        self.contains_hash(hash)
    }

    /// Returns the encoded filter size, including its fixed header.
    #[must_use]
    pub fn size(&self) -> usize {
        self.m / 8 + HEADER_LEN
    }

    /// Returns `true` if the hash may be contained.
    ///
    /// Will never have a false negative.
    #[must_use]
    pub fn contains_hash(&self, mut h1: u64) -> bool {
        let mut h2 = secondary_hash(h1);

        for i in 1..=(self.k as u64) {
            let idx = h1 % (self.m as u64);

            #[expect(
                clippy::cast_possible_truncation,
                reason = "filters in a single table tend to be a couple of megabytes of data at most, so easily fits into usize"
            )]
            if !self.has_bit(idx as usize) {
                return false;
            }

            h1 = h1.wrapping_add(h2);
            h2 = h2.wrapping_mul(i);
        }

        true
    }

    /// Returns `true` if the bit at `idx` is `1`.
    fn has_bit(&self, idx: usize) -> bool {
        debug_assert!(idx < self.m);

        #[expect(
            clippy::indexing_slicing,
            reason = "construction validates the bit array length"
        )]
        let byte = self.inner.as_ref()[HEADER_LEN + idx / 8];
        byte & (0b1000_0000_u8 >> (idx % 8)) != 0
    }
}
