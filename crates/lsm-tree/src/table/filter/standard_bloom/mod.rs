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

    /// Returns `true` if the item may be contained.
    ///
    /// Will never have a false negative.
    #[cfg(test)]
    #[must_use]
    fn contains(&self, key: &[u8]) -> bool {
        self.contains_hash(Builder::get_hash(key))
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

#[cfg(test)]
mod tests {
    use nanoid::nanoid;
    use test_log::test;

    use super::*;

    #[test]
    fn filter_bloom_standard_serde_round_trip() -> Result<()> {
        let mut filter = Builder::with_fp_rate(10, 0.0001);

        let keys = &[
            b"item0", b"item1", b"item2", b"item3", b"item4", b"item5", b"item6", b"item7",
            b"item8", b"item9",
        ];

        for key in keys {
            filter.set_with_hash(Builder::get_hash(*key));
        }

        let filter_bytes = filter.build();
        let encoded_size = filter_bytes.len();
        let filter_copy = StandardBloomFilterReader::new(Slice::from(filter_bytes.clone()))?;
        let borrowed = StandardBloomFilterReader::new(filter_bytes.as_slice())?;
        assert_eq!(filter_copy.size(), encoded_size);
        assert_eq!(borrowed.size(), encoded_size);

        for key in keys
            .iter()
            .map(|key| key.as_slice())
            .chain([b"item10".as_slice(), b"absent".as_slice()])
        {
            assert_eq!(filter_copy.contains(key), borrowed.contains(key));
        }

        assert_eq!(filter.k, filter_copy.k);
        assert_eq!(filter.m, filter_copy.m);
        assert!(!filter_copy.contains(b"asdasads"));
        assert!(!filter_copy.contains(b"item10"));
        assert!(!filter_copy.contains(b"cxycxycxy"));

        Ok(())
    }

    #[test]
    fn filter_bloom_standard_rejects_truncated_bits() {
        let filter = Builder::with_fp_rate(10, 0.0001);
        let mut filter_bytes = filter.build();
        filter_bytes.pop();

        assert!(matches!(
            StandardBloomFilterReader::new(filter_bytes),
            Err(Error::InvalidHeader("BloomFilter"))
        ));
    }

    #[test]
    fn filter_bloom_standard_basic() -> Result<()> {
        let mut filter = Builder::with_fp_rate(10, 0.0001);

        let keys = [
            b"item0" as &[u8],
            b"item1",
            b"item2",
            b"item3",
            b"item4",
            b"item5",
            b"item6",
            b"item7",
            b"item8",
            b"item9",
        ];

        for key in &keys {
            filter.set_with_hash(Builder::get_hash(key));
        }

        let filter_bytes = filter.build();
        let filter = StandardBloomFilterReader::new(filter_bytes)?;

        for key in &keys {
            assert!(filter.contains(key));
        }

        assert!(!filter.contains(b"asdasdasdasdasdasdasd"));

        Ok(())
    }

    #[test]
    fn filter_bloom_standard_bpk() -> Result<()> {
        let item_count = 1_000;
        let bpk = 5.0;

        let mut filter = Builder::with_bpk(item_count, bpk);

        for key in (0..item_count).map(|_| nanoid!()) {
            let key = key.as_bytes();

            filter.set_with_hash(Builder::get_hash(key));
        }

        let filter_bytes = filter.build();
        let filter = StandardBloomFilterReader::new(filter_bytes)?;

        let mut false_positives = 0;

        for key in (0..item_count).map(|_| nanoid!()) {
            let key = key.as_bytes();

            if filter.contains(key) {
                false_positives += 1;
            }
        }

        #[expect(clippy::cast_precision_loss)]
        let fpr = false_positives as f32 / item_count as f32;
        assert!(fpr < 0.13);

        Ok(())
    }

    #[test]
    fn filter_bloom_standard_fpr() -> Result<()> {
        let item_count = 100_000;
        let wanted_fpr = 0.1;

        let mut filter = Builder::with_fp_rate(item_count, wanted_fpr);

        for key in (0..item_count).map(|_| nanoid!()) {
            let key = key.as_bytes();

            filter.set_with_hash(Builder::get_hash(key));
        }

        let filter_bytes = filter.build();
        let filter = StandardBloomFilterReader::new(filter_bytes)?;

        let mut false_positives = 0;

        for key in (0..item_count).map(|_| nanoid!()) {
            let key = key.as_bytes();

            if filter.contains(key) {
                false_positives += 1;
            }
        }

        #[expect(clippy::cast_precision_loss)]
        let fpr = false_positives as f32 / item_count as f32;
        assert!(fpr > 0.05);
        assert!(fpr < 0.13);

        Ok(())
    }

    #[test]
    fn filter_bloom_standard_fpr_2() -> Result<()> {
        let item_count = 100_000;
        let wanted_fpr = 0.5;

        let mut filter = Builder::with_fp_rate(item_count, wanted_fpr);

        for key in (0..item_count).map(|_| nanoid!()) {
            let key = key.as_bytes();

            filter.set_with_hash(Builder::get_hash(key));
        }

        let filter_bytes = filter.build();
        let filter = StandardBloomFilterReader::new(filter_bytes)?;

        let mut false_positives = 0;

        for key in (0..item_count).map(|_| nanoid!()) {
            let key = key.as_bytes();

            if filter.contains(key) {
                false_positives += 1;
            }
        }

        #[expect(clippy::cast_precision_loss)]
        let fpr = false_positives as f32 / item_count as f32;
        assert!(fpr > 0.45);
        assert!(fpr < 0.55);

        Ok(())
    }
}
